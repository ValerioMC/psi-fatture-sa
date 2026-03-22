use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyRevenue {
    pub month: i64,
    pub month_name: String,
    pub revenue: f64,
    pub invoice_count: i64,
}
