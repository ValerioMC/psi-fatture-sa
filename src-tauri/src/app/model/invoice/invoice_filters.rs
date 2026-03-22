use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceFilters {
    pub year: Option<i64>,
    pub status: Option<String>,
    pub client_id: Option<i64>,
    pub search: Option<String>,
}
