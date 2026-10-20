//! Rules an invoice change must respect beyond its own fields: the year's numbering,
//! unique and in date order. What the Sistema TS forbids lives in `ts`.

use sea_orm::ConnectionTrait;

use crate::app::common::AppError;
use crate::app::model::invoice::invoice_number::parse_stored_number;
use crate::app::repository::invoice::invoice_repository;
use crate::app::service::support::italian_format::date_long;

/// A number is used once per year: "007" and "7" count as the same number.
pub(super) async fn ensure_number_free(
    db: &impl ConnectionTrait,
    year: i64,
    number: i64,
    exclude_id: i64,
) -> Result<(), AppError> {
    if !invoice_repository::invoice_number_taken(db, year, number, exclude_id).await? {
        return Ok(());
    }
    Err(AppError::Conflict(format!(
        "Numero fattura {number} già utilizzato nel {year}: scegli un numero libero"
    )))
}

/// Numbers must follow dates: a higher number never carries an earlier date.
pub(super) async fn ensure_chronological(
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
    let clash_number = parse_stored_number(&clash.invoice_number)?;
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

#[cfg(test)]
#[path = "invoice_guard_test.rs"]
mod tests;
