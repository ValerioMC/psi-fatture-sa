//! What the production Sistema TS forbids on an invoice whose expense document it holds.
//! The invoice service knows nothing of the Sistema TS: these run before it.

use sea_orm::ConnectionTrait;

use crate::app::common::AppError;
use crate::app::model::invoice::invoice_number::parse_stored_number;
use crate::app::model::invoice::{Invoice, InvoiceStatus};
use crate::app::model::ts::TsEnvironment;
use crate::app::repository::invoice::invoice_repository;
use crate::app::repository::ts::ts_submission_repository;

/// Deleting the invoice would orphan the data: the transmission is cancelled first.
pub async fn ensure_invoice_removable(
    db: &impl ConnectionTrait,
    invoice_id: i64,
) -> Result<(), AppError> {
    if !held_by_sistema_ts(db, invoice_id).await? {
        return Ok(());
    }
    Err(AppError::Conflict(
        "Fattura trasmessa al Sistema TS: annulla la trasmissione prima di eliminarla".to_string(),
    ))
}

/// A transmitted invoice keeps the id the Sistema TS knows it by and stays paid;
/// amounts and patient may change, then a replacement realigns the Sistema TS.
pub async fn ensure_invoice_update_allowed(
    db: &impl ConnectionTrait,
    current: &Invoice,
    number: i64,
    issue_date: &str,
    status: &InvoiceStatus,
) -> Result<(), AppError> {
    if !held_by_sistema_ts(db, current.id).await? {
        return Ok(());
    }
    let current_number = parse_stored_number(&current.invoice_number)?;
    if current_number != number || current.issue_date != issue_date {
        return Err(AppError::Conflict(
            "Fattura trasmessa al Sistema TS: numero e data non si possono cambiare. \
             Per correggerli annulla prima la trasmissione"
                .to_string(),
        ));
    }
    ensure_stays_paid(status, &current.invoice_number)
}

/// A bulk status change may not move a transmitted invoice off "paid".
pub async fn ensure_invoice_status_allowed(
    db: &impl ConnectionTrait,
    invoice_ids: &[i64],
    status: &InvoiceStatus,
) -> Result<(), AppError> {
    if *status == InvoiceStatus::Paid {
        return Ok(());
    }
    for id in invoice_ids {
        if held_by_sistema_ts(db, *id).await? {
            let invoice = invoice_repository::require(db, *id).await?;
            ensure_stays_paid(status, &invoice.invoice_number)?;
        }
    }
    Ok(())
}

/// Test-environment data never binds the invoice: only production counts.
async fn held_by_sistema_ts(db: &impl ConnectionTrait, invoice_id: i64) -> Result<bool, AppError> {
    ts_submission_repository::invoice_holds_ts_data(db, invoice_id, TsEnvironment::Produzione).await
}

fn ensure_stays_paid(status: &InvoiceStatus, invoice_number: &str) -> Result<(), AppError> {
    if *status == InvoiceStatus::Paid {
        return Ok(());
    }
    Err(AppError::Conflict(format!(
        "La fattura n. {invoice_number} è registrata al Sistema TS come pagata: \
         annulla prima la trasmissione per cambiarne lo stato"
    )))
}

#[cfg(test)]
#[path = "ts_invoice_guard_test.rs"]
mod tests;
