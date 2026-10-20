//! How each attempt moves a queued submission. A claimed submission is a
//! call in flight; a failure without a verdict puts it back with exponential
//! backoff, and only a Sistema TS verdict ends it.

use chrono::{Duration, NaiveDateTime};
use sea_orm::{ActiveValue::Set, ConnectionTrait, IntoActiveModel, TransactionTrait};

use crate::app::common::timestamp;
use crate::app::common::AppError;
use crate::app::entity::ts_submission;
use crate::app::model::ts::{
    TsDocumentId, TsOperation, TsOutcome, TsSubmission, TsSubmissionStatus,
};
use crate::app::repository::ts::ts_submission_repository;

const RETRY_BASE_SECONDS: i64 = 60;
const RETRY_MAX_SECONDS: i64 = 6 * 60 * 60;

/// Queued submissions whose next attempt is due at `now`, oldest first.
pub async fn due(
    db: &impl ConnectionTrait,
    now: NaiveDateTime,
) -> Result<Vec<TsSubmission>, AppError> {
    ts_submission_repository::find_due(db, &timestamp::format(now)).await
}

/// Marks the submission as in flight under `document`; false when another
/// dispatcher got there first or it is no longer queued.
pub async fn claim(
    db: &impl ConnectionTrait,
    id: i64,
    document: &TsDocumentId,
    fingerprint: Option<String>,
    now: NaiveDateTime,
) -> Result<bool, AppError> {
    ts_submission_repository::claim(db, id, document, fingerprint, &timestamp::format(now)).await
}

/// Puts an in-flight submission back in the queue after a failure that never
/// reached a verdict, postponing the next attempt.
pub async fn release_for_retry(
    db: &impl ConnectionTrait,
    id: i64,
    error: &str,
    now: NaiveDateTime,
) -> Result<TsSubmission, AppError> {
    let row = ts_submission_repository::require(db, id).await?;
    transition_guard(&row, TsSubmissionStatus::NonInviata)?;

    let attempts = row.attempt_count;
    let stamp = timestamp::format(now);
    let mut active = row.into_active_model();
    active.status = Set(TsSubmissionStatus::NonInviata.as_str().to_owned());
    active.last_error = Set(Some(error.to_owned()));
    active.next_attempt_at = Set(timestamp::format(now + retry_delay(attempts)));
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
) -> Result<TsSubmission, AppError> {
    let row = ts_submission_repository::require(db, id).await?;
    transition_guard(&row, TsSubmissionStatus::Scartata)?;

    let stamp = timestamp::format(now);
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
) -> Result<TsSubmission, AppError>
where
    C: ConnectionTrait + TransactionTrait,
{
    let next = if outcome.accepted {
        TsSubmissionStatus::Accettata
    } else {
        TsSubmissionStatus::Scartata
    };
    let stamp = timestamp::format(now);
    let tx = db.begin().await?;

    let row = ts_submission_repository::require(&tx, id).await?;
    transition_guard(&row, next)?;
    let retired_target = retired_target(&row, outcome)?;
    store_verdict(&tx, row, next, outcome, &stamp).await?;
    if let Some((target_id, retired)) = retired_target {
        retire_target(&tx, target_id, retired, &stamp).await?;
    }

    tx.commit().await?;
    ts_submission_repository::load(db, id).await
}

/// At start-up, nothing can still be in flight: a previous run stopped mid-call.
pub async fn requeue_interrupted(
    db: &impl ConnectionTrait,
    now: NaiveDateTime,
) -> Result<u64, AppError> {
    ts_submission_repository::requeue_in_flight(db, &timestamp::format(now)).await
}

/// The submission an accepted replacement or cancellation supersedes, with the status it moves to.
fn retired_target(
    row: &ts_submission::Model,
    outcome: &TsOutcome,
) -> Result<Option<(i64, TsSubmissionStatus)>, AppError> {
    let operation = TsOperation::parse(&row.operation)?;
    Ok(match (outcome.accepted, row.target_submission_id) {
        (true, Some(target_id)) => operation
            .target_status_on_acceptance()
            .map(|retired| (target_id, retired)),
        _ => None,
    })
}

async fn store_verdict(
    db: &impl ConnectionTrait,
    row: ts_submission::Model,
    next: TsSubmissionStatus,
    outcome: &TsOutcome,
    stamp: &str,
) -> Result<(), AppError> {
    let mut active = row.into_active_model();
    active.status = Set(next.as_str().to_owned());
    active.protocol = Set(outcome.protocol.clone());
    active.outcome_code = Set(outcome.code.clone());
    active.outcome_message = Set(outcome.message.clone());
    active.last_error = Set(None);
    active.resolved_at = Set(Some(stamp.to_owned()));
    if outcome.accepted {
        active.sent_at = Set(Some(stamp.to_owned()));
    }
    active.updated_at = Set(stamp.to_owned());
    ts_submission_repository::update(db, active).await?;
    Ok(())
}

async fn retire_target(
    db: &impl ConnectionTrait,
    target_id: i64,
    retired: TsSubmissionStatus,
    stamp: &str,
) -> Result<(), AppError> {
    let target = ts_submission_repository::require(db, target_id).await?;
    transition_guard(&target, retired)?;
    let mut active = target.into_active_model();
    active.status = Set(retired.as_str().to_owned());
    active.updated_at = Set(stamp.to_owned());
    ts_submission_repository::update(db, active).await?;
    Ok(())
}

/// Delay before the next attempt: 1 min doubling per attempt, capped at 6 hours.
fn retry_delay(attempt_count: i64) -> Duration {
    let exponent = attempt_count.saturating_sub(1).clamp(0, 20) as u32;
    let seconds = RETRY_BASE_SECONDS.saturating_mul(1_i64 << exponent);
    Duration::seconds(seconds.min(RETRY_MAX_SECONDS))
}

fn transition_guard(row: &ts_submission::Model, next: TsSubmissionStatus) -> Result<(), AppError> {
    let current = TsSubmissionStatus::parse(&row.status)?;
    if !current.can_transition_to(next) {
        return Err(AppError::Invalid(format!(
            "Trasmissione STS {}: passaggio da {} a {} non consentito",
            row.id,
            current.as_str(),
            next.as_str()
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "ts_transition_service_test.rs"]
mod tests;
