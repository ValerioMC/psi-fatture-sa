//! How an invoice number is stored: zero-padded to three digits, so 7 is "007".

use sea_orm::DbErr;

use crate::app::common::AppError;

pub fn format_invoice_number(number: i64) -> String {
    format!("{number:03}")
}

/// Stored numbers are written only by `format_invoice_number`, so a non-numeric one is corrupt data.
pub fn parse_stored_number(invoice_number: &str) -> Result<i64, AppError> {
    invoice_number.parse().map_err(|_| {
        AppError::Database(DbErr::Custom(format!(
            "il numero della fattura «{invoice_number}» salvato in archivio non è numerico"
        )))
    })
}

#[cfg(test)]
#[path = "invoice_number_test.rs"]
mod tests;
