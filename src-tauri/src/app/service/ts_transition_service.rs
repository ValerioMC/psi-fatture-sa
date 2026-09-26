//! How each attempt moves a queued submission. A claimed submission is a
//! call in flight; a failure without a verdict puts it back with exponential
//! backoff, and only a Sistema TS verdict ends it.

use chrono::{Duration, NaiveDateTime};
use sea_orm::{ActiveValue::Set, ConnectionTrait, IntoActiveModel, TransactionTrait};

use crate::app::entity::ts_submission;
use crate::app::model::ts::{
    TsDocumentId, TsOperation, TsOutcome, TsSubmission, TsSubmissionStatus,
};
use crate::app::repository::ts_submission_repository;

const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const RETRY_BASE_SECONDS: i64 = 60;
const RETRY_MAX_SECONDS: i64 = 6 * 60 * 60;

/// Queued submissions whose next attempt is due at `now`, oldest first.
pub async fn due(
    db: &impl ConnectionTrait,
    now: NaiveDateTime,
) -> Result<Vec<TsSubmission>, String> {
    ts_submission_repository::find_due(db, &format_timestamp(now)).await
}

/// Marks the submission as in flight under `document`; false when another
/// dispatcher got there first or it is no longer queued.
pub async fn claim(
    db: &impl ConnectionTrait,
    id: i64,
    document: &TsDocumentId,
    now: NaiveDateTime,
) -> Result<bool, String> {
    ts_submission_repository::claim(db, id, document, &format_timestamp(now)).await
}

/// Puts an in-flight submission back in the queue after a failure that never
/// reached a verdict, postponing the next attempt.
pub async fn release_for_retry(
    db: &impl ConnectionTrait,
    id: i64,
    error: &str,
    now: NaiveDateTime,
) -> Result<TsSubmission, String> {
    let row = ts_submission_repository::find_row(db, id).await?;
    transition_guard(&row, TsSubmissionStatus::NonInviata)?;

    let attempts = row.attempt_count;
    let stamp = format_timestamp(now);
    let mut active = row.into_active_model();
    active.status = Set(TsSubmissionStatus::NonInviata.as_str().to_owned());
    active.last_error = Set(Some(error.to_owned()));
    active.next_attempt_at = Set(format_timestamp(now + retry_delay(attempts)));
    active.updated_at = Set(stamp);
    ts_submission_repository::update(db, active).await?;
    ts_submission_repository::load(db, id).await
}

/// Ends a queued submission that cannot be sent as it stands, such as an
/// invoice that lost its payment date; the reason is kept as the outcome.
pub async fn reject_locally(
    db: &impl ConnectionTrait,
    id: i64,
    reason: &str,
    now: NaiveDateTime,
) -> Result<TsSubmission, String> {
    let row = ts_submission_repository::find_row(db, id).await?;
    transition_guard(&row, TsSubmissionStatus::Scartata)?;

    let stamp = format_timestamp(now);
    let mut active = row.into_active_model();
    active.status = Set(TsSubmissionStatus::Scartata.as_str().to_owned());
    active.outcome_code = Set(Some("LOCALE".to_owned()));
    active.outcome_message = Set(Some(reason.to_owned()));
    active.resolved_at = Set(Some(stamp.clone()));
    active.updated_at = Set(stamp);
    ts_submission_repository::update(db, active).await?;
    ts_submission_repository::load(db, id).await
}

/// Applies the Sistema TS verdict to an in-flight submission. An accepted
/// replacement or cancellation retires its target in the same transaction.
pub async fn record_outcome<C>(
    db: &C,
    id: i64,
    outcome: &TsOutcome,
    now: NaiveDateTime,
) -> Result<TsSubmission, String>
where
    C: ConnectionTrait + TransactionTrait,
{
    let next = if outcome.accepted {
        TsSubmissionStatus::Accettata
    } else {
        TsSubmissionStatus::Scartata
    };
    let stamp = format_timestamp(now);
    let tx = db.begin().await.map_err(|e| e.to_string())?;

    let row = ts_submission_repository::find_row(&tx, id).await?;
    transition_guard(&row, next)?;
    let operation = TsOperation::parse(&row.operation)?;
    let target_id = row.target_submission_id;

    let mut active = row.into_active_model();
    active.status = Set(next.as_str().to_owned());
    active.protocol = Set(outcome.protocol.clone());
    active.outcome_code = Set(outcome.code.clone());
    active.outcome_message = Set(outcome.message.clone());
    active.last_error = Set(None);
    active.resolved_at = Set(Some(stamp.clone()));
    if outcome.accepted {
        active.sent_at = Set(Some(stamp.clone()));
    }
    active.updated_at = Set(stamp.clone());
    ts_submission_repository::update(&tx, active).await?;

    if let (true, Some(target_id), Some(retired)) = (
        outcome.accepted,
        target_id,
        operation.target_status_on_acceptance(),
    ) {
        let target = ts_submission_repository::find_row(&tx, target_id).await?;
        transition_guard(&target, retired)?;
        let mut target_active = target.into_active_model();
        target_active.status = Set(retired.as_str().to_owned());
        target_active.updated_at = Set(stamp);
        ts_submission_repository::update(&tx, target_active).await?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    ts_submission_repository::load(db, id).await
}

/// At start-up, nothing can still be in flight: a previous run stopped mid-call.
pub async fn requeue_interrupted(
    db: &impl ConnectionTrait,
    now: NaiveDateTime,
) -> Result<u64, String> {
    ts_submission_repository::requeue_in_flight(db, &format_timestamp(now)).await
}

/// Delay before the next attempt: 1 min doubling per attempt, capped at 6 hours.
fn retry_delay(attempt_count: i64) -> Duration {
    let exponent = attempt_count.saturating_sub(1).clamp(0, 20) as u32;
    let seconds = RETRY_BASE_SECONDS.saturating_mul(1_i64 << exponent);
    Duration::seconds(seconds.min(RETRY_MAX_SECONDS))
}

pub fn format_timestamp(moment: NaiveDateTime) -> String {
    moment.format(TIMESTAMP_FORMAT).to_string()
}

fn transition_guard(row: &ts_submission::Model, next: TsSubmissionStatus) -> Result<(), String> {
    let current = TsSubmissionStatus::parse(&row.status)?;
    if !current.can_transition_to(next) {
        return Err(format!(
            "Trasmissione STS {}: passaggio da {} a {} non consentito",
            row.id,
            current.as_str(),
            next.as_str()
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::app::service::ts_submission_service::{
        self as queue,
        tests::{force_status, setup},
    };

    pub(crate) fn at(value: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(value, TIMESTAMP_FORMAT).unwrap()
    }

    fn document() -> TsDocumentId {
        TsDocumentId {
            vat_number: "12345678903".to_string(),
            issue_date: "2026-03-01".to_string(),
            number: "1".to_string(),
        }
    }

    fn verdict(accepted: bool) -> TsOutcome {
        TsOutcome {
            accepted,
            protocol: accepted.then(|| "99260926001866680".to_string()),
            code: (!accepted).then(|| "S017".to_string()),
            message: Some("esito".to_string()),
        }
    }

    async fn claimed(db: &sea_orm::DatabaseConnection, id: i64, now: NaiveDateTime) {
        assert!(claim(db, id, &document(), now).await.unwrap());
    }

    #[test]
    fn backoff_doubles_from_one_minute_and_caps_at_six_hours() {
        assert_eq!(retry_delay(1), Duration::seconds(60));
        assert_eq!(retry_delay(2), Duration::seconds(120));
        assert_eq!(retry_delay(5), Duration::seconds(960));
        assert_eq!(retry_delay(9), Duration::seconds(15_360));
        assert_eq!(retry_delay(10), Duration::seconds(RETRY_MAX_SECONDS));
        assert_eq!(retry_delay(1_000), Duration::seconds(RETRY_MAX_SECONDS));
        assert_eq!(retry_delay(0), Duration::seconds(60));
    }

    #[tokio::test]
    async fn claim_is_won_once_and_records_the_document_id() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let now = at("2099-01-01 10:00:00");

        assert!(claim(&db, queued.id, &document(), now).await.unwrap());
        assert!(!claim(&db, queued.id, &document(), now).await.unwrap());

        let row = ts_submission_repository::load(&db, queued.id)
            .await
            .unwrap();
        assert_eq!(row.status, TsSubmissionStatus::Inviata);
        assert_eq!(row.attempt_count, 1);
        assert_eq!(row.document, Some(document()));
        assert!(due(&db, now).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn failed_call_goes_back_to_the_queue_with_backoff() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let now = at("2099-01-01 10:00:00");
        claimed(&db, queued.id, now).await;

        let released = release_for_retry(&db, queued.id, "timeout", now)
            .await
            .unwrap();
        assert_eq!(released.status, TsSubmissionStatus::NonInviata);
        assert_eq!(released.last_error.as_deref(), Some("timeout"));
        assert_eq!(released.next_attempt_at, "2099-01-01 10:01:00");
        assert!(due(&db, at("2099-01-01 10:00:30"))
            .await
            .unwrap()
            .is_empty());
        assert_eq!(due(&db, at("2099-01-01 10:01:00")).await.unwrap().len(), 1);

        claimed(&db, queued.id, at("2099-01-01 10:01:00")).await;
        let again = release_for_retry(&db, queued.id, "dns", at("2099-01-01 10:01:00"))
            .await
            .unwrap();
        assert_eq!(again.next_attempt_at, "2099-01-01 10:03:00");
    }

    #[tokio::test]
    async fn acceptance_records_protocol_and_clears_the_last_error() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let now = at("2099-01-01 10:00:00");
        claimed(&db, queued.id, now).await;
        release_for_retry(&db, queued.id, "timeout", now)
            .await
            .unwrap();
        claimed(&db, queued.id, now).await;

        let accepted = record_outcome(&db, queued.id, &verdict(true), now)
            .await
            .unwrap();
        assert_eq!(accepted.status, TsSubmissionStatus::Accettata);
        assert_eq!(accepted.protocol.as_deref(), Some("99260926001866680"));
        assert!(accepted.last_error.is_none());
        assert_eq!(accepted.sent_at.as_deref(), Some("2099-01-01 10:00:00"));
    }

    #[tokio::test]
    async fn rejection_keeps_the_code() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let now = at("2099-01-01 10:00:00");
        claimed(&db, queued.id, now).await;
        let rejected = record_outcome(&db, queued.id, &verdict(false), now)
            .await
            .unwrap();
        assert_eq!(rejected.status, TsSubmissionStatus::Scartata);
        assert_eq!(rejected.outcome_code.as_deref(), Some("S017"));
        assert!(rejected.sent_at.is_none());
    }

    #[tokio::test]
    async fn local_rejection_ends_a_queued_submission() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let rejected = reject_locally(&db, queued.id, "manca la data", at("2099-01-01 10:00:00"))
            .await
            .unwrap();
        assert_eq!(rejected.status, TsSubmissionStatus::Scartata);
        assert_eq!(rejected.outcome_code.as_deref(), Some("LOCALE"));
    }

    #[tokio::test]
    async fn refuses_transitions_out_of_order() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let now = at("2099-01-01 10:00:00");
        assert!(record_outcome(&db, queued.id, &verdict(true), now)
            .await
            .is_err());
        assert!(release_for_retry(&db, queued.id, "x", now).await.is_err());

        force_status(&db, queued.id, TsSubmissionStatus::Accettata).await;
        assert!(!claim(&db, queued.id, &document(), now).await.unwrap());
        assert!(reject_locally(&db, queued.id, "x", now).await.is_err());
    }

    #[tokio::test]
    async fn accepted_cancellation_retires_its_target() {
        let db = setup().await;
        let now = at("2099-01-01 10:00:00");
        let original = queue::enqueue_invio(&db, 1).await.unwrap();
        force_status(&db, original.id, TsSubmissionStatus::Accettata).await;

        let cancellation = queue::enqueue_cancellation(&db, original.id).await.unwrap();
        claimed(&db, cancellation.id, now).await;
        record_outcome(&db, cancellation.id, &verdict(true), now)
            .await
            .unwrap();

        let original = ts_submission_repository::load(&db, original.id)
            .await
            .unwrap();
        assert_eq!(original.status, TsSubmissionStatus::Annullata);
    }

    #[tokio::test]
    async fn accepted_replacement_supersedes_its_target_and_rejection_leaves_it() {
        let db = setup().await;
        let now = at("2099-01-01 10:00:00");
        let original = queue::enqueue_invio(&db, 1).await.unwrap();
        force_status(&db, original.id, TsSubmissionStatus::Accettata).await;

        let rejected = queue::enqueue_replacement(&db, original.id).await.unwrap();
        claimed(&db, rejected.id, now).await;
        record_outcome(&db, rejected.id, &verdict(false), now)
            .await
            .unwrap();
        let still = ts_submission_repository::load(&db, original.id)
            .await
            .unwrap();
        assert_eq!(still.status, TsSubmissionStatus::Accettata);

        let replacement = queue::enqueue_replacement(&db, original.id).await.unwrap();
        claimed(&db, replacement.id, now).await;
        record_outcome(&db, replacement.id, &verdict(true), now)
            .await
            .unwrap();
        let original = ts_submission_repository::load(&db, original.id)
            .await
            .unwrap();
        assert_eq!(original.status, TsSubmissionStatus::Sostituita);
    }

    #[tokio::test]
    async fn start_up_requeues_calls_left_in_flight() {
        let db = setup().await;
        let queued = queue::enqueue_invio(&db, 1).await.unwrap();
        let now = at("2099-01-01 10:00:00");
        claimed(&db, queued.id, now).await;

        assert_eq!(requeue_interrupted(&db, now).await.unwrap(), 1);
        let row = ts_submission_repository::load(&db, queued.id)
            .await
            .unwrap();
        assert_eq!(row.status, TsSubmissionStatus::NonInviata);
        assert_eq!(due(&db, now).await.unwrap().len(), 1);
    }
}
