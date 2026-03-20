/// Tauri commands for dashboard analytics.
use crate::commands::invoices::get_invoice;
use crate::db;
use crate::domain::models::{DashboardData, MonthlyRevenue};
use rusqlite::params;

const MONTH_NAMES: [&str; 12] = [
    "Gennaio", "Febbraio", "Marzo", "Aprile", "Maggio", "Giugno",
    "Luglio", "Agosto", "Settembre", "Ottobre", "Novembre", "Dicembre",
];

#[tauri::command]
pub fn get_dashboard(year: i64) -> Result<DashboardData, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let total_revenue: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_due), 0) FROM invoices WHERE year = ?1 AND status != 'cancelled'",
            params![year],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let paid_revenue: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_due), 0) FROM invoices WHERE year = ?1 AND status = 'paid'",
            params![year],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let unpaid_revenue: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_due), 0) FROM invoices
             WHERE year = ?1 AND status IN ('issued', 'overdue')",
            params![year],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let total_invoices: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM invoices WHERE year = ?1 AND status != 'cancelled'",
            params![year],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let paid_invoices: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM invoices WHERE year = ?1 AND status = 'paid'",
            params![year],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let draft_invoices: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM invoices WHERE year = ?1 AND status = 'draft'",
            params![year],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let monthly_revenue = build_monthly_revenue(&conn, year)?;

    let recent_ids: Vec<i64> = {
        let mut stmt = conn
            .prepare(
                "SELECT id FROM invoices WHERE year = ?1 AND status != 'cancelled'
                 ORDER BY issue_date DESC LIMIT 5",
            )
            .map_err(|e| e.to_string())?;
        let ids: Vec<i64> = stmt
            .query_map(params![year], |r| r.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        ids
    };

    let recent_invoices = recent_ids
        .into_iter()
        .map(get_invoice)
        .filter_map(|r| r.ok())
        .collect();

    Ok(DashboardData {
        year,
        total_revenue,
        paid_revenue,
        unpaid_revenue,
        total_invoices,
        paid_invoices,
        draft_invoices,
        monthly_revenue,
        recent_invoices,
    })
}

fn build_monthly_revenue(
    conn: &rusqlite::Connection,
    year: i64,
) -> Result<Vec<MonthlyRevenue>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT CAST(strftime('%m', issue_date) AS INTEGER) as month,
                    COALESCE(SUM(total_due), 0) as revenue,
                    COUNT(*) as cnt
             FROM invoices
             WHERE year = ?1 AND status = 'paid'
             GROUP BY month ORDER BY month",
        )
        .map_err(|e| e.to_string())?;

    let db_rows: std::collections::HashMap<i64, (f64, i64)> = stmt
        .query_map(params![year], |r| Ok((r.get::<_, i64>(0)?, r.get(1)?, r.get(2)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .map(|(m, rev, cnt)| (m, (rev, cnt)))
        .collect();

    Ok((1i64..=12)
        .map(|m| {
            let (revenue, invoice_count) = db_rows.get(&m).copied().unwrap_or((0.0, 0));
            MonthlyRevenue {
                month: m,
                month_name: MONTH_NAMES[(m - 1) as usize].to_string(),
                revenue,
                invoice_count,
            }
        })
        .collect())
}
