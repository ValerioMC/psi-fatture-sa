use serde::{Deserialize, Serialize};

use super::payment_method::PaymentMethod;

/// Input for generating monthly invoices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateMonthlyInput {
    pub year: i64,
    pub month: i64,
    pub client_ids: Vec<i64>,
    pub payment_method: PaymentMethod,
    pub apply_enpap: bool,
}
