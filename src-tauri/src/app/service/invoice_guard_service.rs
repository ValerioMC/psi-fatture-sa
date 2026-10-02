//! Rules an invoice change must respect beyond its own fields: the numbering
//! order of the year and the data the production Sistema TS already holds.

use sea_orm::ConnectionTrait;

use crate::app::common::AppError;
use crate::app::model::invoice::{Invoice, InvoiceStatus};
use crate::app::model::ts::TsEnvironment;
use crate::app::repository::invoice::invoice_repository;
use crate::app::repository::ts::ts_submission_repository;
use crate::app::service::italian_format::date_long;

/// Numbers must follow dates: a higher number never carries an earlier date.
pub async fn ensure_chronological(
    db: &impl ConnectionTrait,
    year: i64,
    number: i64,
    issue_date: &str,
    exclude_id: i64,
) -> Result<(), AppError> {
    let Some(clash) =
        invoice_repository::find_out_of_order(db, year, number, issue_date, exclude_id).await?
    else {
        return Ok(());
    };
    let clash_number = clash.invoice_number.parse::<i64>().unwrap_or_default();
    let date = date_long(&clash.issue_date);
    let hint = if clash_number < number {
        format!("usa una data dal {date} in poi")
    } else {
        format!("usa una data fino al {date}")
    };
    Err(AppError::Conflict(format!(
        "La fattura n. {} è del {date}: la numerazione deve seguire l'ordine delle date, {hint}",
        clash.invoice_number
    )))
}

/// Whether the production Sistema TS holds this invoice's expense document.
pub async fn held_by_sistema_ts(
    db: &impl ConnectionTrait,
    invoice_id: i64,
) -> Result<bool, AppError> {
    ts_submission_repository::invoice_holds_ts_data(db, invoice_id, TsEnvironment::Produzione).await
}

/// A transmitted invoice keeps the id the Sistema TS knows it by and stays paid;
/// amounts and patient may change, then a replacement realigns the Sistema TS.
pub async fn ensure_ts_allows_update(
    db: &impl ConnectionTrait,
    current: &Invoice,
    number: i64,
    issue_date: &str,
    status: &InvoiceStatus,
) -> Result<(), AppError> {
    if !held_by_sistema_ts(db, current.id).await? {
        return Ok(());
    }
    let current_number = current.invoice_number.parse::<i64>().unwrap_or_default();
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
pub async fn ensure_ts_allows_status(
    db: &impl ConnectionTrait,
    invoice_ids: &[i64],
    status: &InvoiceStatus,
) -> Result<(), AppError> {
    if *status == InvoiceStatus::Paid {
        return Ok(());
    }
    for id in invoice_ids {
        if held_by_sistema_ts(db, *id).await? {
            let invoice = invoice_repository::load_invoice(db, *id).await?;
            ensure_stays_paid(status, &invoice.invoice_number)?;
        }
    }
    Ok(())
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
#[path = "invoice_guard_service_test.rs"]
mod tests;
