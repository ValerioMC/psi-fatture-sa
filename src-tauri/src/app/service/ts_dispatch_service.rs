//! Sends what is due in the queue, one call at a time. A verdict ends a
//! submission; anything else leaves it queued. An unreachable service or a
//! refused login stops the pass: the rest would fail the same way.

use std::sync::LazyLock;

use chrono::NaiveDateTime;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;

use crate::app::model::ts::{
    TsCallResponse, TsDispatchSummary, TsDocumentId, TsExpenseDocument, TsOperation, TsOutcome,
    TsSettings, TsSubmission,
};
use crate::app::repository::secret_store::SecretStore;
use crate::app::repository::sistema_ts::gateway::{SistemaTsGateway, TsGatewayError, TsSession};
use crate::app::repository::ts_submission_repository;
use crate::app::service::{
    ts_credential_service, ts_document_service, ts_settings_service,
    ts_submission_service::document_id_of, ts_transition_service as transition,
};

/// One pass at a time: the worker and a manual "send now" never overlap.
static DISPATCH_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Codes that mean an earlier attempt already went through: the document is
/// already there (insert) or already gone (cancel).
const ALREADY_INSERTED: &str = "S017";
const ALREADY_CANCELLED: &str = "S022";

enum Request {
    Insert(TsExpenseDocument),
    Update(TsExpenseDocument),
    Cancel(TsDocumentId),
}

impl Request {
    fn document_id(&self) -> &TsDocumentId {
        match self {
            Request::Insert(document) | Request::Update(document) => &document.id,
            Request::Cancel(id) => id,
        }
    }
}

enum Attempt {
    Accepted,
    Rejected,
    Retrying,
    StopPass(String),
    Skipped,
}

/// Sends every submission due at `now` in the configured environment.
pub async fn dispatch_due(
    db: &DatabaseConnection,
    store: &dyn SecretStore,
    gateway: &dyn SistemaTsGateway,
    now: NaiveDateTime,
) -> Result<TsDispatchSummary, String> {
    let _pass = DISPATCH_LOCK.lock().await;
    let settings = ts_settings_service::get(db).await?;
    let (due, elsewhere): (Vec<TsSubmission>, Vec<TsSubmission>) = transition::due(db, now)
        .await?
        .into_iter()
        .partition(|s| s.environment == settings.environment);

    let mut summary = TsDispatchSummary {
        waiting_other_environment: elsewhere.len() as u32,
        ..Default::default()
    };
    if due.is_empty() {
        return Ok(summary);
    }
    let session = match ts_credential_service::session(store, &settings) {
        Ok(session) => session,
        Err(reason) => {
            summary.blocked = Some(reason);
            return Ok(summary);
        }
    };

    for submission in due {
        match dispatch_one(db, gateway, &session, &settings, &submission, now).await? {
            Attempt::Accepted => summary.accepted += 1,
            Attempt::Rejected => summary.rejected += 1,
            Attempt::Retrying => summary.retrying += 1,
            Attempt::Skipped => {}
            Attempt::StopPass(reason) => {
                summary.retrying += 1;
                summary.blocked = Some(reason);
                break;
            }
        }
    }
    Ok(summary)
}

async fn dispatch_one(
    db: &DatabaseConnection,
    gateway: &dyn SistemaTsGateway,
    session: &TsSession,
    settings: &TsSettings,
    submission: &TsSubmission,
    now: NaiveDateTime,
) -> Result<Attempt, String> {
    let request = match prepare(db, settings, submission).await {
        Ok(request) => request,
        Err(reason) => {
            transition::reject_locally(db, submission.id, &reason, now).await?;
            return Ok(Attempt::Rejected);
        }
    };
    if !transition::claim(db, submission.id, request.document_id(), now).await? {
        return Ok(Attempt::Skipped);
    }

    let result = match &request {
        Request::Insert(document) => gateway.insert(session, document).await,
        Request::Update(document) => gateway.update(session, document).await,
        Request::Cancel(id) => gateway.cancel(session, id).await,
    };
    settle(db, submission, result, now).await
}

/// Builds the call from the invoice as it is now; follow-ups reuse the id
/// their target was sent under.
async fn prepare(
    db: &DatabaseConnection,
    settings: &TsSettings,
    submission: &TsSubmission,
) -> Result<Request, String> {
    let target_id = match submission.target_submission_id {
        Some(target) => {
            let row = ts_submission_repository::find_row(db, target).await?;
            Some(document_id_of(&row).ok_or("La trasmissione originale non ha un identificativo")?)
        }
        None => None,
    };
    match submission.operation {
        TsOperation::Invio => {
            ts_document_service::build(db, submission.invoice_id, &settings.vat_number, None)
                .await
                .map(Request::Insert)
        }
        TsOperation::Sostituzione => {
            let id = target_id.ok_or("Sostituzione senza trasmissione originale")?;
            let vat_number = id.vat_number.clone();
            ts_document_service::build(db, submission.invoice_id, &vat_number, Some(id))
                .await
                .map(Request::Update)
        }
        TsOperation::Annullamento => target_id
            .map(Request::Cancel)
            .ok_or_else(|| "Annullamento senza trasmissione originale".to_string()),
    }
}

async fn settle(
    db: &DatabaseConnection,
    submission: &TsSubmission,
    result: Result<TsCallResponse, TsGatewayError>,
    now: NaiveDateTime,
) -> Result<Attempt, String> {
    let response = match result {
        Ok(response) => response,
        Err(error) => {
            let reason = error.to_string();
            transition::release_for_retry(db, submission.id, &reason, now).await?;
            return Ok(match error {
                TsGatewayError::Authentication | TsGatewayError::Network(_) => {
                    Attempt::StopPass(reason)
                }
                _ => Attempt::Retrying,
            });
        }
    };

    if response.accepted() {
        transition::record_outcome(db, submission.id, &outcome_of(&response, true), now).await?;
        return Ok(Attempt::Accepted);
    }
    if response.is_retryable_rejection() {
        let reason = response
            .summary_message()
            .unwrap_or_else(|| "Errore temporaneo".to_string());
        transition::release_for_retry(db, submission.id, &reason, now).await?;
        return Ok(if response.is_credential_rejection() {
            Attempt::StopPass(reason)
        } else {
            Attempt::Retrying
        });
    }
    if submission.attempt_count > 0 && confirms_earlier_attempt(submission.operation, &response) {
        let outcome = TsOutcome {
            accepted: true,
            protocol: None,
            code: response.summary_code(),
            message: Some("Già registrato dal Sistema TS con un tentativo precedente".to_string()),
        };
        transition::record_outcome(db, submission.id, &outcome, now).await?;
        return Ok(Attempt::Accepted);
    }
    transition::record_outcome(db, submission.id, &outcome_of(&response, false), now).await?;
    Ok(Attempt::Rejected)
}

/// After a call whose answer was lost, the retry meets its own earlier success.
fn confirms_earlier_attempt(operation: TsOperation, response: &TsCallResponse) -> bool {
    match operation {
        TsOperation::Invio => response.has_code(ALREADY_INSERTED),
        TsOperation::Annullamento => response.has_code(ALREADY_CANCELLED),
        TsOperation::Sostituzione => false,
    }
}

fn outcome_of(response: &TsCallResponse, accepted: bool) -> TsOutcome {
    TsOutcome {
        accepted,
        protocol: response.protocol.clone(),
        code: response.summary_code(),
        message: response.summary_message(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::model::ts::{TsSubmissionStatus, TsVatTreatment};
    use crate::app::repository::secret_store::testing::InMemorySecretStore;
    use crate::app::repository::secret_store::SecretKind;
    use crate::app::repository::sistema_ts::gateway::testing::{
        accepted, rejected, FakeGateway, RecordedCall,
    };
    use crate::app::service::ts_submission_service::{self as queue, tests::setup};
    use crate::app::service::ts_transition_service::tests::at;
    use sea_orm::ConnectionTrait;

    fn store() -> InMemorySecretStore {
        let store = InMemorySecretStore::default();
        store.write(SecretKind::TsPassword, "Salve123").unwrap();
        store.write(SecretKind::TsPincode, "3489543096").unwrap();
        store
    }

    async fn run(db: &DatabaseConnection, gateway: &FakeGateway, now: &str) -> TsDispatchSummary {
        dispatch_due(db, &store(), gateway, at(now)).await.unwrap()
    }

    async fn load(db: &DatabaseConnection, id: i64) -> TsSubmission {
        ts_submission_repository::load(db, id).await.unwrap()
    }

    #[tokio::test]
    async fn sends_a_queued_invoice_and_records_the_protocol() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let gateway = FakeGateway::default();
        gateway.script(Ok(accepted("99260926001866680")));

        let summary = run(&db, &gateway, "2099-01-01 10:00:00").await;
        assert_eq!(summary.accepted, 1);

        let sent = load(&db, queued.id).await;
        assert_eq!(sent.status, TsSubmissionStatus::Accettata);
        assert_eq!(sent.protocol.as_deref(), Some("99260926001866680"));
        assert_eq!(sent.document.as_ref().unwrap().vat_number, "65498732105");

        let calls = gateway.recorded();
        let [RecordedCall::Insert(document)] = calls.as_slice() else {
            panic!("expected one insert");
        };
        assert_eq!(
            document.citizen_fiscal_code.as_deref(),
            Some("RSSMRA80A41H501Y")
        );
        assert_eq!(document.items[0].amount, 81.64);
        assert_eq!(
            document.items[0].vat,
            TsVatTreatment::Natura("N2.2".to_string())
        );
    }

    #[tokio::test]
    async fn missing_credentials_block_the_pass_without_touching_the_queue() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let gateway = FakeGateway::default();
        let summary = dispatch_due(
            &db,
            &InMemorySecretStore::default(),
            &gateway,
            at("2099-01-01 10:00:00"),
        )
        .await
        .unwrap();
        assert!(summary.blocked.unwrap().contains("password"));
        assert!(gateway.recorded().is_empty());
        assert_eq!(
            load(&db, queued.id).await.status,
            TsSubmissionStatus::NonInviata
        );
    }

    #[tokio::test]
    async fn unreachable_service_stops_the_pass() {
        let db = setup().await;
        db.execute_unprepared(
            "UPDATE invoices SET status = 'paid', paid_date = '2026-03-07' WHERE id = 3",
        )
        .await
        .unwrap();
        queue::enqueue_invio(&db, 1).await.unwrap();
        let second = queue::enqueue_invio(&db, 3).await.unwrap();
        let gateway = FakeGateway::default();
        gateway.script(Err(TsGatewayError::Network("timed out".to_string())));

        let summary = run(&db, &gateway, "2099-01-01 10:00:00").await;
        assert!(summary.blocked.unwrap().contains("non raggiungibile"));
        assert_eq!(gateway.recorded().len(), 1);
        assert_eq!(load(&db, second.id).await.attempt_count, 0);
    }

    #[tokio::test]
    async fn network_errors_and_ws99_are_retried_later() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let gateway = FakeGateway::default();
        gateway.script(Err(TsGatewayError::Network("timeout".to_string())));
        gateway.script(Ok(rejected("WS99")));

        assert_eq!(run(&db, &gateway, "2099-01-01 10:00:00").await.retrying, 1);
        let first = load(&db, queued.id).await;
        assert_eq!(first.status, TsSubmissionStatus::NonInviata);
        assert!(first.last_error.unwrap().contains("timeout"));

        assert_eq!(
            run(&db, &gateway, "2099-01-01 10:00:30").await,
            TsDispatchSummary::default()
        );
        assert_eq!(run(&db, &gateway, "2099-01-01 10:01:00").await.retrying, 1);
        assert_eq!(run(&db, &gateway, "2099-01-01 10:03:00").await.accepted, 1);
        assert_eq!(load(&db, queued.id).await.attempt_count, 3);
    }

    #[tokio::test]
    async fn refused_login_stops_the_pass_and_keeps_everything_queued() {
        let db = setup().await;
        db.execute_unprepared(
            "UPDATE invoices SET status = 'paid', paid_date = '2026-03-07' WHERE id = 3",
        )
        .await
        .unwrap();
        let first = queue::enqueue_invio(&db, 1).await.unwrap();
        let second = queue::enqueue_invio(&db, 3).await.unwrap();
        let gateway = FakeGateway::default();
        gateway.script(Err(TsGatewayError::Authentication));

        let summary = run(&db, &gateway, "2099-01-01 10:00:00").await;
        assert!(summary.blocked.unwrap().contains("password"));
        assert_eq!(gateway.recorded().len(), 1);
        assert_eq!(
            load(&db, first.id).await.status,
            TsSubmissionStatus::NonInviata
        );
        assert_eq!(
            load(&db, second.id).await.status,
            TsSubmissionStatus::NonInviata
        );
    }

    #[tokio::test]
    async fn wrong_pincode_is_a_credential_problem_not_a_rejection() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let gateway = FakeGateway::default();
        gateway.script(Ok(rejected("005")));
        let summary = run(&db, &gateway, "2099-01-01 10:00:00").await;
        assert!(summary.blocked.is_some());
        assert_eq!(
            load(&db, queued.id).await.status,
            TsSubmissionStatus::NonInviata
        );
    }

    #[tokio::test]
    async fn document_errors_are_final() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let gateway = FakeGateway::default();
        gateway.script(Ok(rejected("WS19")));
        assert_eq!(run(&db, &gateway, "2099-01-01 10:00:00").await.rejected, 1);
        let rejected = load(&db, queued.id).await;
        assert_eq!(rejected.status, TsSubmissionStatus::Scartata);
        assert_eq!(rejected.outcome_code.as_deref(), Some("WS19"));
    }

    #[tokio::test]
    async fn duplicate_after_a_lost_answer_counts_as_accepted() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let gateway = FakeGateway::default();
        gateway.script(Err(TsGatewayError::Network("reset".to_string())));
        gateway.script(Ok(rejected("S017")));

        run(&db, &gateway, "2099-01-01 10:00:00").await;
        assert_eq!(run(&db, &gateway, "2099-01-01 10:01:00").await.accepted, 1);
        assert_eq!(
            load(&db, queued.id).await.status,
            TsSubmissionStatus::Accettata
        );
    }

    #[tokio::test]
    async fn duplicate_on_a_first_attempt_is_a_rejection() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let gateway = FakeGateway::default();
        gateway.script(Ok(rejected("S017")));
        run(&db, &gateway, "2099-01-01 10:00:00").await;
        assert_eq!(
            load(&db, queued.id).await.status,
            TsSubmissionStatus::Scartata
        );
    }

    #[tokio::test]
    async fn replacement_and_cancellation_reuse_the_original_document_id() {
        let db = setup().await;
        let original = queue::enqueue_invio(&db, 1).await.unwrap();
        let gateway = FakeGateway::default();
        run(&db, &gateway, "2099-01-01 10:00:00").await;
        let sent_id = load(&db, original.id).await.document.unwrap();

        db.execute_unprepared(
            "UPDATE invoices SET invoice_number = '1-bis', total_gross = 90 WHERE id = 1",
        )
        .await
        .unwrap();
        let replacement = queue::enqueue_replacement(&db, original.id).await.unwrap();
        run(&db, &gateway, "2099-01-01 10:05:00").await;
        assert_eq!(
            load(&db, original.id).await.status,
            TsSubmissionStatus::Sostituita
        );

        let cancellation = queue::enqueue_cancellation(&db, replacement.id)
            .await
            .unwrap();
        run(&db, &gateway, "2099-01-01 10:10:00").await;
        assert_eq!(
            load(&db, replacement.id).await.status,
            TsSubmissionStatus::Annullata
        );
        assert_eq!(
            load(&db, cancellation.id).await.status,
            TsSubmissionStatus::Accettata
        );

        let calls = gateway.recorded();
        let RecordedCall::Update(updated) = &calls[1] else {
            panic!("expected an update")
        };
        assert_eq!(updated.id, sent_id);
        assert_eq!(updated.items[0].amount, 90.0);
        assert_eq!(calls[2], RecordedCall::Cancel(sent_id));
    }

    #[tokio::test]
    async fn invoice_changed_since_queueing_is_rejected_locally() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        db.execute_unprepared("UPDATE invoices SET status = 'issued' WHERE id = 1")
            .await
            .unwrap();
        let gateway = FakeGateway::default();
        assert_eq!(run(&db, &gateway, "2099-01-01 10:00:00").await.rejected, 1);
        assert!(gateway.recorded().is_empty());
        let rejected = load(&db, queued.id).await;
        assert_eq!(rejected.outcome_code.as_deref(), Some("LOCALE"));
    }

    #[tokio::test]
    async fn submissions_for_another_environment_wait() {
        let db = setup().await;
        queue::enqueue_invio(&db, 1).await.unwrap();
        db.execute_unprepared("UPDATE ts_settings SET environment = 'produzione'")
            .await
            .unwrap();
        let gateway = FakeGateway::default();
        let summary = run(&db, &gateway, "2099-01-01 10:00:00").await;
        assert_eq!(summary.waiting_other_environment, 1);
        assert!(gateway.recorded().is_empty());
    }
}
