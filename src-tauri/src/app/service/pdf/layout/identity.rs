//! The right column of the header: "FATTURA", the number and its dates, right-aligned.

use super::style::{ACCENT, HAIRLINE, INK, RIGHT, RULE, SUBTLE};
use crate::app::model::invoice::Invoice;
use crate::app::service::pdf::block::baseline;
use crate::app::service::pdf::{Block, Face, InvoiceDocument, TextStyle};
use crate::app::service::support::italian_format::date_long;

/// Returns the column's bottom.
pub(super) fn invoice_identity(block: &mut Block, document: &InvoiceDocument) -> f32 {
    block.text_right(
        RIGHT,
        baseline(0.0, 7.0, 10.0),
        TextStyle::new(Face::Bold, 7.0, ACCENT).tracked(1.5),
        "FATTURA",
    );
    let mut y = 10.0 + 3.0;
    let number_width = invoice_number(block, &document.invoice.invoice_number, y);
    y += 30.0 + 7.5;
    date_rows(block, &document.invoice, y, number_width)
}

/// "N. 12" in display type; returns its width, the narrowest the date rows may be.
fn invoice_number(block: &mut Block, number: &str, top: f32) -> f32 {
    let number_style = TextStyle::new(Face::Display, 30.0, INK).tracked(-0.75);
    let prefix = TextStyle::new(Face::DisplayLight, 15.0, SUBTLE);
    let number_width = block.width(&number_style, number);
    let prefix_width = block.width(&prefix, "N.") + 1.5;
    let number_baseline = top + 30.0 * 0.8;
    block.text(
        RIGHT - number_width - prefix_width,
        number_baseline,
        prefix,
        "N.",
    );
    block.text(RIGHT - number_width, number_baseline, number_style, number);
    number_width + prefix_width
}

/// Issue and due date under a rule as wide as the widest row or the number; returns the bottom.
fn date_rows(block: &mut Block, invoice: &Invoice, mut y: f32, min_width: f32) -> f32 {
    let label = TextStyle::new(Face::SemiBold, 7.0, SUBTLE).tracked(0.4);
    let value = TextStyle::new(Face::SemiBold, 8.0, INK);
    let mut rows = vec![("DATA EMISSIONE", date_long(&invoice.issue_date))];
    if let Some(due) = invoice.due_date.as_deref().filter(|due| !due.is_empty()) {
        rows.push(("SCADENZA", date_long(due)));
    }
    let row_width = rows
        .iter()
        .map(|(name, date)| block.width(&label, name) + 6.0 + block.width(&value, date))
        .fold(min_width, f32::max);
    block.line((RIGHT - row_width, y), (RIGHT, y), RULE, HAIRLINE);
    y += 6.0;
    for (name, date) in &rows {
        let row_baseline = baseline(y, 8.0, 12.0);
        let date_width = block.width(&value, date);
        block.text_right(RIGHT, row_baseline, value, date);
        block.text_right(RIGHT - date_width - 6.0, row_baseline, label, name);
        y += 12.0 + 2.25;
    }
    y
}
