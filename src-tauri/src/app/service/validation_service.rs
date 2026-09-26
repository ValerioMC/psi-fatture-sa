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

/// Validates an Italian codice fiscale when provided. Accepts the
/// 16-character personal format (with checksum verification, including
/// omocodia variants) and the 11-digit format used by legal entities.
/// Mirrors `validateCodiceFiscale` in `src/utils/validation.ts` so the
/// frontend and backend reject the same inputs with the same messages.
pub fn validate_fiscal_code(value: &str) -> Result<(), String> {
    let v = value.trim().to_uppercase();
    if v.is_empty() {
        return Ok(());
    }
    if v.len() == 11 && v.chars().all(|c| c.is_ascii_digit()) {
        return validate_vat_number(&v);
    }
    if v.chars().count() != 16 {
        return Err("Il codice fiscale deve avere 16 caratteri".to_string());
    }
    if !v.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("Il codice fiscale contiene caratteri non validi".to_string());
    }
    if !cf_shape_is_valid(&cf_de_omocodia(&v)) {
        return Err("Formato codice fiscale non valido".to_string());
    }
    let chars: Vec<char> = v.chars().collect();
    if chars[15] != cf_control_char(&chars) {
        return Err("Codice fiscale non valido (carattere di controllo errato)".to_string());
    }
    Ok(())
}

/// Validates an Italian VAT number when provided (11 digits, Luhn checksum).
/// Mirrors `validatePartitaIva` in `src/utils/validation.ts`.
pub fn validate_vat_number(value: &str) -> Result<(), String> {
    let v = value.trim();
    if v.is_empty() {
        return Ok(());
    }
    if v.len() != 11 || !v.chars().all(|c| c.is_ascii_digit()) {
        return Err("La partita IVA deve essere composta da 11 cifre".to_string());
    }
    let sum: u32 = v
        .bytes()
        .enumerate()
        .map(|(i, b)| {
            let mut digit = u32::from(b - b'0');
            if i % 2 == 1 {
                digit *= 2;
                if digit > 9 {
                    digit -= 9;
                }
            }
            digit
        })
        .sum();
    if !sum.is_multiple_of(10) {
        return Err("Partita IVA non valida (cifra di controllo errata)".to_string());
    }
    Ok(())
}

/// Checks the LLLLLL DD L DD L DDD L layout of a codice fiscale
/// (after omocodia letters have been restored to digits).
fn cf_shape_is_valid(cf: &[char]) -> bool {
    cf.len() == 16
        && cf.iter().enumerate().all(|(i, c)| match i {
            0..=5 | 8 | 11 | 15 => c.is_ascii_uppercase(),
            _ => c.is_ascii_digit(),
        })
}

/// Restores omocodia-substituted letters to digits in the numeric positions.
fn cf_de_omocodia(cf: &str) -> Vec<char> {
    const NUMERIC_POSITIONS: [usize; 7] = [6, 7, 9, 10, 12, 13, 14];
    let mut chars: Vec<char> = cf.chars().collect();
    for &pos in &NUMERIC_POSITIONS {
        if let Some(digit) = cf_omocodia_digit(chars[pos]) {
            chars[pos] = digit;
        }
    }
    chars
}

fn cf_omocodia_digit(c: char) -> Option<char> {
    match c {
        'L' => Some('0'),
        'M' => Some('1'),
        'N' => Some('2'),
        'P' => Some('3'),
        'Q' => Some('4'),
        'R' => Some('5'),
        'S' => Some('6'),
        'T' => Some('7'),
        'U' => Some('8'),
        'V' => Some('9'),
        _ => None,
    }
}

/// Computes the expected control character from the first 15 characters.
fn cf_control_char(chars: &[char]) -> char {
    let sum: u32 = chars[..15]
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            if i % 2 == 0 {
                cf_odd_value(c)
            } else {
                cf_even_value(c)
            }
        })
        .sum();
    char::from(b'A' + (sum % 26) as u8)
}

fn cf_even_value(c: char) -> u32 {
    if c.is_ascii_digit() {
        c as u32 - '0' as u32
    } else {
        c as u32 - 'A' as u32
    }
}

fn cf_odd_value(c: char) -> u32 {
    match c {
        '0' | 'A' => 1,
        '1' | 'B' => 0,
        '2' | 'C' => 5,
        '3' | 'D' => 7,
        '4' | 'E' => 9,
        '5' | 'F' => 13,
        '6' | 'G' => 15,
        '7' | 'H' => 17,
        '8' | 'I' => 19,
        '9' | 'J' => 21,
        'K' => 2,
        'L' => 4,
        'M' => 18,
        'N' => 20,
        'O' => 11,
        'P' => 3,
        'Q' => 6,
        'R' => 8,
        'S' => 12,
        'T' => 14,
        'U' => 16,
        'V' => 10,
        'W' => 22,
        'X' => 25,
        'Y' => 24,
        'Z' => 23,
        _ => 0,
    }
}

#[cfg(test)]
#[path = "validation_service_test.rs"]
mod tests;
