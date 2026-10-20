//! Changes to an invoice the Sistema TS may hold. The `ts` rules are checked first, then
//! `invoice_service` applies the change, so the invoice domain never depends on `ts`.

use sea_orm::DatabaseConnection;

use crate::app::common::{AppError, Clock};
use crate::app::model::invoice::{BulkUpdateStatusInput, Invoice, UpdateInvoiceInput};
use crate::app::service::invoice::invoice_service;
use crate::app::service::ts::ts_submission_service;

/// Invalid input is refused before a Sistema TS lock is reported.
pub async fn update(
    db: &DatabaseConnection,
    clock: &dyn Clock,
    input: UpdateInvoiceInput,
) -> Result<Invoice, AppError> {
    let checked = invoice_service::check_update(db, input).await?;
    ts_submission_service::ensure_invoice_update_allowed(
        db,
        checked.current(),
        checked.number(),
        &checked.input().issue_date,
        &checked.input().status,
    )
    .await?;
    invoice_service::update(db, clock, checked).await
}

pub async fn remove(db: &DatabaseConnection, id: i64) -> Result<(), AppError> {
    ts_submission_service::ensure_invoice_removable(db, id).await?;
    invoice_service::remove(db, id).await
}

pub async fn bulk_update_status(
    db: &DatabaseConnection,
    clock: &dyn Clock,
    input: BulkUpdateStatusInput,
) -> Result<u64, AppError> {
    ts_submission_service::ensure_invoice_status_allowed(db, &input.ids, &input.status).await?;
    invoice_service::bulk_update_status(db, clock, input).await
}

#[cfg(test)]
#[path = "invoice_change_service_test.rs"]
mod tests;
