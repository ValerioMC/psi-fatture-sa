//! The checks an invoice's own fields must pass before any rule reads the archive.

use chrono::NaiveDate;

use crate::app::common::AppError;
use crate::app::model::invoice::InvoiceLineInput;
use crate::app::model::tax::InvoiceLineData;
use crate::app::service::support::validation as validate;

/// Validates issue/due dates of an invoice, returning the parsed issue date.
pub(super) fn validate_invoice_dates(
    issue_date: &str,
    due_date: Option<&str>,
) -> Result<NaiveDate, AppError> {
    let issue = validate::parse_iso_date(issue_date, "Data emissione")?;
    if let Some(due) = due_date.filter(|d| !d.is_empty()) {
        let due = validate::parse_iso_date(due, "Data scadenza")?;
        if due < issue {
            return Err(AppError::Invalid(
                "La data di scadenza precede la data di emissione".to_string(),
            ));
        }
    }
    Ok(issue)
}

/// Validates the line items of an invoice.
pub(super) fn validate_invoice_lines(lines: &[InvoiceLineInput]) -> Result<(), AppError> {
    if lines.is_empty() {
        return Err(AppError::Invalid(
            "La fattura deve contenere almeno una riga".to_string(),
        ));
    }
    for (index, line) in lines.iter().enumerate() {
        validate_invoice_line(index + 1, line)?;
    }
    ensure_positive_total(lines)
}

/// `row` is the 1-based position the user sees in the form.
fn validate_invoice_line(row: usize, line: &InvoiceLineInput) -> Result<(), AppError> {
    let invalid = |problem: &str| Err(AppError::Invalid(format!("Riga {row}: {problem}")));
    if line.description.trim().is_empty() {
        return invalid("descrizione obbligatoria");
    }
    if line.quantity < 1 {
        return invalid("la quantità deve essere almeno 1");
    }
    if !line.unit_price.is_finite() || line.unit_price < 0.0 {
        return invalid("prezzo unitario non valido");
    }
    if !line.vat_rate.is_finite() || !(0.0..=100.0).contains(&line.vat_rate) {
        return invalid("aliquota IVA non valida (0-100)");
    }
    if line
        .amount_override
        .is_some_and(|amount| !amount.is_finite() || amount < 0.0)
    {
        return invalid("importo non valido");
    }
    Ok(())
}

fn ensure_positive_total(lines: &[InvoiceLineInput]) -> Result<(), AppError> {
    let total: f64 = lines
        .iter()
        .map(|line| InvoiceLineData::from(line).net_amount())
        .sum();
    if total <= 0.0 {
        return Err(AppError::Invalid(
            "L'importo della fattura deve essere maggiore di zero".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "invoice_validation_test.rs"]
mod tests;
