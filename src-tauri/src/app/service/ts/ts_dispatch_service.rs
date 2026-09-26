//! Sends what is due in the queue, one call at a time. A verdict ends a
//! submission; anything else leaves it queued. An unreachable service or a
//! refused login stops the pass: the rest would fail the same way.

use std::sync::LazyLock;

use chrono::NaiveDateTime;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;

use super::{TsDispatchAttempt, TsDispatchRequest};
use crate::app::model::ts::{
    TsCallResponse, TsDispatchSummary, TsOperation, TsOutcome, TsSettings, TsSubmission,
};
use crate::app::repository::secret::SecretStore;
use crate::app::repository::ts::sistema_ts::{SistemaTsGateway, TsGatewayError, TsSession};
use crate::app::repository::ts::ts_submission_repository;
use crate::app::service::ts::{
    ts_credential_service, ts_document_service, ts_settings_service,
    ts_submission_service::document_id_of, ts_transition_service as transition,
};

/// One pass at a time: the worker and a manual "send now" never overlap.
static DISPATCH_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Codes that mean an earlier attempt already went through: the document is
/// already there (insert) or already gone (cancel).
const ALREADY_INSERTED: &str = "S017";
const ALREADY_CANCELLED: &str = "S022";

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
            TsDispatchAttempt::Accepted => summary.accepted += 1,
            TsDispatchAttempt::Rejected => summary.rejected += 1,
            TsDispatchAttempt::Retrying => summary.retrying += 1,
            TsDispatchAttempt::Skipped => {}
            TsDispatchAttempt::StopPass(reason) => {
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
) -> Result<TsDispatchAttempt, String> {
    let request = match prepare(db, settings, submission).await {
        Ok(request) => request,
        Err(reason) => {
            transition::reject_locally(db, submission.id, &reason, now).await?;
            return Ok(TsDispatchAttempt::Rejected);
        }
    };
    if !transition::claim(db, submission.id, request.document_id(), now).await? {
        return Ok(TsDispatchAttempt::Skipped);
    }

    let result = match &request {
        TsDispatchRequest::Insert(document) => gateway.insert(session, document).await,
        TsDispatchRequest::Update(document) => gateway.update(session, document).await,
        TsDispatchRequest::Cancel(id) => gateway.cancel(session, id).await,
    };
    settle(db, submission, result, now).await
}

/// Builds the call from the invoice as it is now; follow-ups reuse the id
/// their target was sent under.
async fn prepare(
    db: &DatabaseConnection,
    settings: &TsSettings,
    submission: &TsSubmission,
) -> Result<TsDispatchRequest, String> {
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
                .map(TsDispatchRequest::Insert)
        }
        TsOperation::Sostituzione => {
            let id = target_id.ok_or("Sostituzione senza trasmissione originale")?;
            let vat_number = id.vat_number.clone();
            ts_document_service::build(db, submission.invoice_id, &vat_number, Some(id))
                .await
                .map(TsDispatchRequest::Update)
        }
        TsOperation::Annullamento => target_id
            .map(TsDispatchRequest::Cancel)
            .ok_or_else(|| "Annullamento senza trasmissione originale".to_string()),
    }
}

async fn settle(
    db: &DatabaseConnection,
    submission: &TsSubmission,
    result: Result<TsCallResponse, TsGatewayError>,
    now: NaiveDateTime,
) -> Result<TsDispatchAttempt, String> {
    let response = match result {
        Ok(response) => response,
        Err(error) => {
            let reason = error.to_string();
            transition::release_for_retry(db, submission.id, &reason, now).await?;
            return Ok(match error {
                TsGatewayError::Authentication
                | TsGatewayError::Network(_)
                | TsGatewayError::UnavailableEnvironment => TsDispatchAttempt::StopPass(reason),
                _ => TsDispatchAttempt::Retrying,
            });
        }
    };

    if response.accepted() {
        transition::record_outcome(db, submission.id, &outcome_of(&response, true), now).await?;
        return Ok(TsDispatchAttempt::Accepted);
    }
    if response.is_retryable_rejection() {
        let reason = response
            .summary_message()
            .unwrap_or_else(|| "Errore temporaneo".to_string());
        transition::release_for_retry(db, submission.id, &reason, now).await?;
        return Ok(if response.is_credential_rejection() {
            TsDispatchAttempt::StopPass(reason)
        } else {
            TsDispatchAttempt::Retrying
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
        return Ok(TsDispatchAttempt::Accepted);
    }
    transition::record_outcome(db, submission.id, &outcome_of(&response, false), now).await?;
    Ok(TsDispatchAttempt::Rejected)
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
#[path = "ts_dispatch_service_test.rs"]
mod tests;
