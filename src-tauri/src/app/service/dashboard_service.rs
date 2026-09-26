use std::collections::HashMap;

use sea_orm::DatabaseConnection;

use crate::app::model::dashboard::{DashboardData, MonthTotal, MonthlyRevenue};
use crate::app::model::invoice::Invoice;
use crate::app::repository::dashboard_repository;
use crate::app::repository::invoice::invoice_repository;

const MONTH_NAMES: [&str; 12] = [
    "Gennaio",
    "Febbraio",
    "Marzo",
    "Aprile",
    "Maggio",
    "Giugno",
    "Luglio",
    "Agosto",
    "Settembre",
    "Ottobre",
    "Novembre",
    "Dicembre",
];

/// Returns aggregated dashboard analytics for the given year.
pub async fn get(db: &DatabaseConnection, year: i64) -> Result<DashboardData, String> {
    let month_totals = dashboard_repository::paid_totals_by_month(db, year).await?;
    Ok(DashboardData {
        year,
        total_revenue: dashboard_repository::revenue(db, year).await?,
        total_net_revenue: dashboard_repository::net_revenue(db, year).await?,
        paid_revenue: dashboard_repository::paid_revenue(db, year).await?,
        unpaid_revenue: dashboard_repository::unpaid_revenue(db, year).await?,
        total_invoices: dashboard_repository::invoice_count(db, year).await?,
        paid_invoices: dashboard_repository::paid_invoice_count(db, year).await?,
        draft_invoices: dashboard_repository::draft_invoice_count(db, year).await?,
        monthly_revenue: monthly_revenue(&month_totals),
        recent_invoices: recent_invoices(db, year).await?,
    })
}

/// All twelve months, zero where nothing was paid.
fn monthly_revenue(totals: &[MonthTotal]) -> Vec<MonthlyRevenue> {
    let by_month: HashMap<i64, &MonthTotal> = totals.iter().map(|t| (t.month, t)).collect();
    (1i64..=12)
        .map(|month| {
            let (revenue, invoice_count) = by_month
                .get(&month)
                .map(|t| (t.revenue, t.invoice_count))
                .unwrap_or((0.0, 0));
            MonthlyRevenue {
                month,
                month_name: MONTH_NAMES[(month - 1) as usize].to_string(),
                revenue,
                invoice_count,
            }
        })
        .collect()
}

/// An invoice that fails to load is left out rather than failing the whole dashboard.
async fn recent_invoices(db: &DatabaseConnection, year: i64) -> Result<Vec<Invoice>, String> {
    let mut invoices = Vec::new();
    for id in dashboard_repository::recent_invoice_ids(db, year).await? {
        match invoice_repository::load_invoice(db, id).await {
            Ok(invoice) => invoices.push(invoice),
            Err(error) => {
                log::warn!(target: "dashboard", invoice_id = id, error:% = error; "recent invoice skipped")
            }
        }
    }
    Ok(invoices)
}

#[cfg(test)]
#[path = "dashboard_service_test.rs"]
mod tests;
