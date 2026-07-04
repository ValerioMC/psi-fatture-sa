//! Input validation shared by all services.
//!
//! Every Tauri command input passes through these checks before touching
//! the database, so invalid data is rejected with a clear Italian message
//! instead of being persisted or causing a panic.

use chrono::NaiveDate;

use crate::app::model::invoice::InvoiceLineInput;

const MIN_YEAR: i64 = 2000;
const MAX_YEAR: i64 = 2100;

/// Parses and validates an ISO date string (YYYY-MM-DD).
pub fn parse_iso_date(value: &str, field: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| format!("{field}: data non valida ({value}), atteso formato YYYY-MM-DD"))
}

/// Validates a calendar year within a sane range.
pub fn validate_year(year: i64) -> Result<(), String> {
    if (MIN_YEAR..=MAX_YEAR).contains(&year) {
        Ok(())
    } else {
        Err(format!("Anno non valido: {year}"))
    }
}

/// Validates a calendar month (1-12).
pub fn validate_month(month: i64) -> Result<(), String> {
    if (1..=12).contains(&month) {
        Ok(())
    } else {
        Err(format!("Mese non valido: {month}"))
    }
}

/// Validates a time string in HH:MM format.
pub fn validate_time(value: &str, field: &str) -> Result<(), String> {
    let valid = value.len() == 5
        && value.as_bytes()[2] == b':'
        && value[..2].parse::<u32>().map(|h| h < 24).unwrap_or(false)
        && value[3..].parse::<u32>().map(|m| m < 60).unwrap_or(false);
    if valid {
        Ok(())
    } else {
        Err(format!(
            "{field}: orario non valido ({value}), atteso HH:MM"
        ))
    }
}

/// Validates that a referenced id is a plausible primary key.
pub fn validate_id(id: i64, field: &str) -> Result<(), String> {
    if id > 0 {
        Ok(())
    } else {
        Err(format!("{field}: selezione obbligatoria"))
    }
}

/// Validates issue/due dates of an invoice, returning the parsed issue date.
pub fn validate_invoice_dates(
    issue_date: &str,
    due_date: Option<&str>,
) -> Result<NaiveDate, String> {
    let issue = parse_iso_date(issue_date, "Data emissione")?;
    if let Some(due) = due_date.filter(|d| !d.is_empty()) {
        let due = parse_iso_date(due, "Data scadenza")?;
        if due < issue {
            return Err("La data di scadenza precede la data di emissione".to_string());
        }
    }
    Ok(issue)
}

/// Validates the line items of an invoice.
pub fn validate_invoice_lines(lines: &[InvoiceLineInput]) -> Result<(), String> {
    if lines.is_empty() {
        return Err("La fattura deve contenere almeno una riga".to_string());
    }
    for (idx, l) in lines.iter().enumerate() {
        let row = idx + 1;
        if l.description.trim().is_empty() {
            return Err(format!("Riga {row}: descrizione obbligatoria"));
        }
        if l.quantity < 1 {
            return Err(format!("Riga {row}: la quantità deve essere almeno 1"));
        }
        if !l.unit_price.is_finite() || l.unit_price < 0.0 {
            return Err(format!("Riga {row}: prezzo unitario non valido"));
        }
        if !l.vat_rate.is_finite() || !(0.0..=100.0).contains(&l.vat_rate) {
            return Err(format!("Riga {row}: aliquota IVA non valida (0-100)"));
        }
    }
    Ok(())
}

/// Validates that a mandatory text field is not blank.
pub fn validate_required(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field}: campo obbligatorio"))
    } else {
        Ok(())
    }
}

/// Validates an Italian codice fiscale when provided (16 alphanumeric
/// characters for individuals, 11 digits for legal entities).
pub fn validate_fiscal_code(value: &str) -> Result<(), String> {
    let v = value.trim();
    if v.is_empty() {
        return Ok(());
    }
    let is_person = v.len() == 16 && v.chars().all(|c| c.is_ascii_alphanumeric());
    let is_entity = v.len() == 11 && v.chars().all(|c| c.is_ascii_digit());
    if is_person || is_entity {
        Ok(())
    } else {
        Err(format!("Codice fiscale non valido: {v}"))
    }
}

/// Validates an Italian VAT number when provided (11 digits).
pub fn validate_vat_number(value: &str) -> Result<(), String> {
    let v = value.trim();
    if v.is_empty() {
        return Ok(());
    }
    if v.len() == 11 && v.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(format!("Partita IVA non valida: {v}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_line() -> InvoiceLineInput {
        InvoiceLineInput {
            service_id: None,
            description: "Seduta di psicoterapia".to_string(),
            quantity: 1,
            unit_price: 70.0,
            vat_rate: 0.0,
        }
    }

    #[test]
    fn parses_valid_iso_date() {
        assert!(parse_iso_date("2026-07-04", "test").is_ok());
    }

    #[test]
    fn rejects_malformed_dates() {
        assert!(parse_iso_date("04/07/2026", "test").is_err());
        assert!(parse_iso_date("2026-13-01", "test").is_err());
        assert!(parse_iso_date("", "test").is_err());
    }

    #[test]
    fn validates_month_bounds() {
        assert!(validate_month(1).is_ok());
        assert!(validate_month(12).is_ok());
        assert!(validate_month(0).is_err());
        assert!(validate_month(13).is_err());
    }

    #[test]
    fn validates_year_bounds() {
        assert!(validate_year(2026).is_ok());
        assert!(validate_year(1900).is_err());
    }

    #[test]
    fn validates_time_format() {
        assert!(validate_time("09:30", "test").is_ok());
        assert!(validate_time("23:59", "test").is_ok());
        assert!(validate_time("24:00", "test").is_err());
        assert!(validate_time("9:30", "test").is_err());
        assert!(validate_time("09.30", "test").is_err());
    }

    #[test]
    fn due_date_must_not_precede_issue_date() {
        assert!(validate_invoice_dates("2026-07-04", Some("2026-07-03")).is_err());
        assert!(validate_invoice_dates("2026-07-04", Some("2026-07-04")).is_ok());
        assert!(validate_invoice_dates("2026-07-04", None).is_ok());
        assert!(validate_invoice_dates("2026-07-04", Some("")).is_ok());
    }

    #[test]
    fn rejects_empty_invoice_lines() {
        assert!(validate_invoice_lines(&[]).is_err());
    }

    #[test]
    fn rejects_invalid_line_values() {
        let mut blank_description = valid_line();
        blank_description.description = "  ".to_string();
        assert!(validate_invoice_lines(&[blank_description]).is_err());

        let mut zero_quantity = valid_line();
        zero_quantity.quantity = 0;
        assert!(validate_invoice_lines(&[zero_quantity]).is_err());

        let mut negative_price = valid_line();
        negative_price.unit_price = -1.0;
        assert!(validate_invoice_lines(&[negative_price]).is_err());

        let mut bad_vat = valid_line();
        bad_vat.vat_rate = 101.0;
        assert!(validate_invoice_lines(&[bad_vat]).is_err());

        assert!(validate_invoice_lines(&[valid_line()]).is_ok());
    }

    #[test]
    fn validates_fiscal_code_shapes() {
        assert!(validate_fiscal_code("RSSMRA80A01H501U").is_ok());
        assert!(validate_fiscal_code("01234567890").is_ok());
        assert!(validate_fiscal_code("").is_ok());
        assert!(validate_fiscal_code("SHORT").is_err());
        assert!(validate_fiscal_code("RSSMRA80A01H501!").is_err());
    }

    #[test]
    fn validates_vat_number_shape() {
        assert!(validate_vat_number("01234567890").is_ok());
        assert!(validate_vat_number("").is_ok());
        assert!(validate_vat_number("0123456789").is_err());
        assert!(validate_vat_number("0123456789A").is_err());
    }
}
