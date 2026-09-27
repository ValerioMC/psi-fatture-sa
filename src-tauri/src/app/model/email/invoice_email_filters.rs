use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct InvoiceEmailFilters {
    pub invoice_id: Option<i64>,
}
