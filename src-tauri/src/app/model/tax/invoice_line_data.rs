use super::money::round_cents;
use crate::app::model::invoice::InvoiceLineInput;

/// The part of an invoice line the totals are computed from.
#[derive(Debug, Clone)]
pub struct InvoiceLineData {
    pub quantity: i64,
    pub unit_price: f64,
    pub vat_rate: f64,
    pub amount_override: Option<f64>,
}

impl InvoiceLineData {
    /// The line's taxable amount: the one typed by hand when there is one,
    /// otherwise quantity × unit price, rounded per line.
    pub fn net_amount(&self) -> f64 {
        round_cents(
            self.amount_override
                .unwrap_or(self.quantity as f64 * self.unit_price),
        )
    }

    pub fn vat_amount(&self) -> f64 {
        round_cents(self.net_amount() * self.vat_rate / 100.0)
    }
}

impl From<&InvoiceLineInput> for InvoiceLineData {
    fn from(line: &InvoiceLineInput) -> Self {
        InvoiceLineData {
            quantity: line.quantity,
            unit_price: line.unit_price,
            vat_rate: line.vat_rate,
            amount_override: line.amount_override,
        }
    }
}
