use serde::{Deserialize, Serialize};

use super::invoice_status::InvoiceStatus;

/// Input for bulk-updating the status of multiple invoices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkUpdateStatusInput {
    pub ids: Vec<i64>,
    pub status: InvoiceStatus,
    pub paid_date: Option<String>,
}
