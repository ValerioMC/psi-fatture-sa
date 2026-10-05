//! Amounts and dates written the way an Italian invoice and its email write
//! them, matching the frontend's `formatCurrency` and `formatDateLong`.

const MONTHS: [&str; 12] = [
    "gennaio",
    "febbraio",
    "marzo",
    "aprile",
    "maggio",
    "giugno",
    "luglio",
    "agosto",
    "settembre",
    "ottobre",
    "novembre",
    "dicembre",
];

/// "1.234,56 €": thousands always grouped, two decimals, rounded half away from zero.
pub fn currency(amount: f64) -> String {
    let cents = (amount.abs() * 100.0).round() as u64;
    let euros = (cents / 100).to_string();
    let mut grouped = String::with_capacity(euros.len() + euros.len() / 3);
    for (index, digit) in euros.chars().enumerate() {
        if index > 0 && (euros.len() - index).is_multiple_of(3) {
            grouped.push('.');
        }
        grouped.push(digit);
    }
    let sign = if amount < 0.0 && cents > 0 { "-" } else { "" };
    format!("{sign}{grouped},{:02} €", cents % 100)
}

/// "20 marzo 2026" from "2026-03-20"; anything unparseable comes back as given.
pub fn date_long(iso_date: &str) -> String {
    let date = iso_date.split('T').next().unwrap_or_default();
    let mut parts = date.split('-');
    let (Some(year), Some(month), Some(day)) = (parts.next(), parts.next(), parts.next()) else {
        return iso_date.to_string();
    };
    let month_name = month
        .parse::<usize>()
        .ok()
        .and_then(|m| m.checked_sub(1))
        .and_then(|index| MONTHS.get(index));
    match (day.parse::<u32>(), month_name) {
        (Ok(day), Some(month_name)) => format!("{day} {month_name} {year}"),
        _ => iso_date.to_string(),
    }
}

#[cfg(test)]
#[path = "italian_format_test.rs"]
mod tests;
