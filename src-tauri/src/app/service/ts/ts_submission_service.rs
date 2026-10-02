//! The user's side of the Sistema TS queue: queueing a first transmission,
//! a replacement or a cancellation of an accepted one, and withdrawing
//! whatever has not left the machine yet. Sending is the worker's job.

use sea_orm::{ActiveValue::Set, ConnectionTrait, DatabaseConnection, TransactionTrait};

use crate::app::common::AppError;
use crate::app::entity::ts_submission::ActiveModel;
use crate::app::model::ts::{
    TsDocumentId, TsEnvironment, TsOperation, TsSubmission, TsSubmissionFilters, TsSubmissionStatus,
};
use crate::app::repository::ts::ts_submission_repository;
use crate::app::service::ts::{ts_document_service, ts_settings_service};
use crate::app::service::validation_service as validate;

const IN_FLIGHT: [TsSubmissionStatus; 2] =
    [TsSubmissionStatus::NonInviata, TsSubmissionStatus::Inviata];

pub async fn list(
    db: &DatabaseConnection,
    filters: TsSubmissionFilters,
) -> Result<Vec<TsSubmission>, AppError> {
    ts_submission_repository::find(db, &filters).await
}

/// Queues the first transmission of a paid invoice to the configured
/// environment, checking now that the document can be built.
pub async fn enqueue_invio(
    db: &DatabaseConnection,
    invoice_id: i64,
) -> Result<TsSubmission, AppError> {
    validate::validate_id(invoice_id, "Fattura")?;
    let tx = db.begin().await?;

    let settings = ts_settings_service::get(&tx).await?;
    if settings.vat_number.trim().is_empty() {
        return Err(AppError::Invalid(
            "Indica la partita IVA del Sistema TS nelle Impostazioni".to_string(),
        ));
    }
    ts_document_service::build(&tx, invoice_id, &settings.vat_number, None).await?;
    ensure_nothing_in_flight(&tx, invoice_id).await?;
    if ts_submission_repository::invoice_has_status(
        &tx,
        invoice_id,
        &[TsSubmissionStatus::Accettata],
        Some(settings.environment),
    )
    .await?
    {
        return Err(AppError::Conflict(
            "Fattura già accettata dal Sistema TS: per correggerla usa la sostituzione".to_string(),
        ));
    }

    let created = ts_submission_repository::insert(
        &tx,
        new_submission(invoice_id, TsOperation::Invio, None, settings.environment),
    )
    .await?;
    tx.commit().await?;
    ts_submission_repository::load(db, created.id).await
}

/// Queues a replacement of an accepted submission with the invoice's current data.
pub async fn enqueue_replacement(
    db: &DatabaseConnection,
    target_submission_id: i64,
) -> Result<TsSubmission, AppError> {
    enqueue_follow_up(db, target_submission_id, TsOperation::Sostituzione).await
}

/// Queues the cancellation of an accepted submission.
pub async fn enqueue_cancellation(
    db: &DatabaseConnection,
    target_submission_id: i64,
) -> Result<TsSubmission, AppError> {
    enqueue_follow_up(db, target_submission_id, TsOperation::Annullamento).await
}

/// Removes a submission that is still waiting in the queue.
pub async fn withdraw(db: &DatabaseConnection, submission_id: i64) -> Result<(), AppError> {
    validate::validate_id(submission_id, "Trasmissione STS")?;
    let row = ts_submission_repository::find_row(db, submission_id).await?;
    if TsSubmissionStatus::parse(&row.status)? != TsSubmissionStatus::NonInviata {
        return Err(AppError::Invalid(
            "Solo le trasmissioni non ancora inviate possono essere ritirate dalla coda"
                .to_string(),
        ));
    }
    ts_submission_repository::delete(db, submission_id).await
}

/// Whether deleting the invoice would orphan data the production Sistema TS holds.
pub async fn blocks_invoice_deletion(
    db: &DatabaseConnection,
    invoice_id: i64,
) -> Result<bool, AppError> {
    ts_submission_repository::invoice_holds_ts_data(db, invoice_id, TsEnvironment::Produzione).await
}

/// Whether the invoice changed since the Sistema TS accepted its data, so a
/// replacement is due. Submissions sent before fingerprints existed never report it.
pub async fn invoice_out_of_date(
    db: &DatabaseConnection,
    invoice_id: i64,
) -> Result<bool, AppError> {
    validate::validate_id(invoice_id, "Fattura")?;
    let Some(row) = ts_submission_repository::find_latest_accepted_document(db, invoice_id).await?
    else {
        return Ok(false);
    };
    let (Some(sent), Some(id)) = (row.document_fingerprint.clone(), document_id_of(&row)) else {
        return Ok(false);
    };
    let vat_number = id.vat_number.clone();
    match ts_document_service::build(db, invoice_id, &vat_number, Some(id)).await {
        Ok(current) => Ok(current.fingerprint() != sent),
        Err(AppError::Invalid(_) | AppError::Conflict(_)) => Ok(true),
        Err(error) => Err(error),
    }
}

// ─── Private helpers ──────────────────────────────────────────────────────────

async fn enqueue_follow_up(
    db: &DatabaseConnection,
    target_submission_id: i64,
    operation: TsOperation,
) -> Result<TsSubmission, AppError> {
    validate::validate_id(target_submission_id, "Trasmissione STS")?;
    let tx = db.begin().await?;

    let target = ts_submission_repository::find_row(&tx, target_submission_id).await?;
    if TsSubmissionStatus::parse(&target.status)? != TsSubmissionStatus::Accettata {
        return Err(AppError::Conflict(format!(
            "Si può chiedere {} solo di una trasmissione accettata",
            follow_up_label(operation)
        )));
    }
    let environment = TsEnvironment::parse(&target.environment)?;
    if operation == TsOperation::Sostituzione {
        let id = document_id_of(&target);
        let vat_number = id
            .as_ref()
            .map(|i| i.vat_number.clone())
            .unwrap_or_default();
        ts_document_service::build(&tx, target.invoice_id, &vat_number, id).await?;
    }
    ensure_nothing_in_flight(&tx, target.invoice_id).await?;

    let created = ts_submission_repository::insert(
        &tx,
        new_submission(target.invoice_id, operation, Some(target.id), environment),
    )
    .await?;
    tx.commit().await?;
    ts_submission_repository::load(db, created.id).await
}

/// The id an accepted submission was sent under, which follow-ups must reuse.
pub(crate) fn document_id_of(
    row: &crate::app::entity::ts_submission::Model,
) -> Option<TsDocumentId> {
    match (
        &row.document_vat_number,
        &row.document_issue_date,
        &row.document_number,
    ) {
        (Some(vat_number), Some(issue_date), Some(number)) => Some(TsDocumentId {
            vat_number: vat_number.clone(),
            issue_date: issue_date.clone(),
            number: number.clone(),
        }),
        _ => None,
    }
}

async fn ensure_nothing_in_flight(
    db: &impl ConnectionTrait,
    invoice_id: i64,
) -> Result<(), AppError> {
    if ts_submission_repository::invoice_has_status(db, invoice_id, &IN_FLIGHT, None).await? {
        return Err(AppError::Conflict(
            "C'è già una trasmissione in corso per questa fattura: attendi l'esito".to_string(),
        ));
    }
    Ok(())
}

fn new_submission(
    invoice_id: i64,
    operation: TsOperation,
    target_submission_id: Option<i64>,
    environment: TsEnvironment,
) -> ActiveModel {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    ActiveModel {
        invoice_id: Set(invoice_id),
        operation: Set(operation.as_str().to_owned()),
        status: Set(TsSubmissionStatus::NonInviata.as_str().to_owned()),
        target_submission_id: Set(target_submission_id),
        environment: Set(environment.as_str().to_owned()),
        attempt_count: Set(0),
        next_attempt_at: Set(now.clone()),
        created_at: Set(now.clone()),
        updated_at: Set(now),
        ..Default::default()
    }
}

fn follow_up_label(operation: TsOperation) -> &'static str {
    match operation {
        TsOperation::Sostituzione => "la sostituzione",
        _ => "l'annullamento",
    }
}

#[cfg(test)]
#[path = "ts_submission_service_test.rs"]
mod tests;
