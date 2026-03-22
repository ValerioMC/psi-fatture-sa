use serde::{Deserialize, Serialize};

use crate::app::model::invoice::Invoice;

use super::monthly_revenue::MonthlyRevenue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub year: i64,
    pub total_revenue: f64,
    pub total_net_revenue: f64,
    pub paid_revenue: f64,
    pub unpaid_revenue: f64,
    pub total_invoices: i64,
    pub paid_invoices: i64,
    pub draft_invoices: i64,
    pub monthly_revenue: Vec<MonthlyRevenue>,
    pub recent_invoices: Vec<Invoice>,
}
