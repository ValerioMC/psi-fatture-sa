/// The part of an invoice line the totals are computed from.
#[derive(Debug, Clone)]
pub struct InvoiceLineData {
    pub quantity: i64,
    pub unit_price: f64,
    pub vat_rate: f64,
}
