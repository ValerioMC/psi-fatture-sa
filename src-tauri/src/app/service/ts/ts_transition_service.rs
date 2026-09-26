//! How each attempt moves a queued submission. A claimed submission is a
//! call in flight; a failure without a verdict puts it back with exponential
//! backoff, and only a Sistema TS verdict ends it.

use chrono::{Duration, NaiveDateTime};
use sea_orm::{ActiveValue::Set, ConnectionTrait, IntoActiveModel, TransactionTrait};

use crate::app::entity::ts_submission;
use crate::app::model::ts::{
    TsDocumentId, TsOperation, TsOutcome, TsSubmission, TsSubmissionStatus,
};
use crate::app::repository::ts::ts_submission_repository;

pub(crate) const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
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
#[path = "ts_transition_service_test.rs"]
mod tests;
