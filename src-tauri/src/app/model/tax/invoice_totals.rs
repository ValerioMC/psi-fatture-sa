use serde::Serialize;

/// Every amount of an invoice, rounded to the cent as it is stored.
#[derive(Debug, Clone, Serialize)]
pub struct InvoiceTotals {
    pub total_net: f64,
    pub total_tax: f64,
    pub contributo_enpap: f64,
    pub ritenuta_acconto: f64,
    pub marca_da_bollo: f64,
    pub total_gross: f64,
    pub total_due: f64,
}
