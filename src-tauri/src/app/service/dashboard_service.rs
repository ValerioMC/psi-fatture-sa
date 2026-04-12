use sea_orm::{DatabaseConnection, FromQueryResult, Statement};

use crate::app::model::dashboard::{DashboardData, MonthlyRevenue};
use crate::app::repository::invoice_repository;

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
    let total_revenue = query_f64(db, "SELECT CAST(COALESCE(SUM(total_due),0) AS REAL) AS val FROM invoices WHERE year=? AND status!='cancelled'", year).await?;
    let total_net_revenue = query_f64(db, "SELECT CAST(COALESCE(SUM(total_net),0) AS REAL) AS val FROM invoices WHERE year=? AND status!='cancelled'", year).await?;
    let paid_revenue = query_f64(db, "SELECT CAST(COALESCE(SUM(total_due),0) AS REAL) AS val FROM invoices WHERE year=? AND status='paid'", year).await?;
    let unpaid_revenue = query_f64(db, "SELECT CAST(COALESCE(SUM(total_due),0) AS REAL) AS val FROM invoices WHERE year=? AND status IN ('issued','overdue')", year).await?;
    let total_invoices = query_i64(
        db,
        "SELECT COUNT(*) AS val FROM invoices WHERE year=? AND status!='cancelled'",
        year,
    )
    .await?;
    let paid_invoices = query_i64(
        db,
        "SELECT COUNT(*) AS val FROM invoices WHERE year=? AND status='paid'",
        year,
    )
    .await?;
    let draft_invoices = query_i64(
        db,
        "SELECT COUNT(*) AS val FROM invoices WHERE year=? AND status='draft'",
        year,
    )
    .await?;

    let monthly_revenue = build_monthly_revenue(db, year).await?;
    let recent_invoices = load_recent_invoices(db, year).await?;

    Ok(DashboardData {
        year,
        total_revenue,
        total_net_revenue,
        paid_revenue,
        unpaid_revenue,
        total_invoices,
        paid_invoices,
        draft_invoices,
        monthly_revenue,
        recent_invoices,
    })
}

// ─── Private helpers ──────────────────────────────────────────────────────────

async fn query_f64(db: &DatabaseConnection, sql: &str, year: i64) -> Result<f64, String> {
    #[derive(FromQueryResult)]
    struct Row {
        val: f64,
    }
    let row = Row::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        sql,
        [year.into()],
    ))
    .one(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row.map(|r| r.val).unwrap_or(0.0))
}

async fn query_i64(db: &DatabaseConnection, sql: &str, year: i64) -> Result<i64, String> {
    #[derive(FromQueryResult)]
    struct Row {
        val: i64,
    }
    let row = Row::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        sql,
        [year.into()],
    ))
    .one(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row.map(|r| r.val).unwrap_or(0))
}

async fn build_monthly_revenue(
    db: &DatabaseConnection,
    year: i64,
) -> Result<Vec<MonthlyRevenue>, String> {
    #[derive(FromQueryResult)]
    struct Row {
        month: i64,
        revenue: f64,
        invoice_count: i64,
    }

    let rows = Row::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT CAST(strftime('%m', issue_date) AS INTEGER) AS month,
                CAST(COALESCE(SUM(total_due), 0) AS REAL) AS revenue,
                COUNT(*) AS invoice_count
         FROM invoices WHERE year=? AND status='paid'
         GROUP BY month ORDER BY month",
        [year.into()],
    ))
    .all(db)
    .await
    .map_err(|e| e.to_string())?;

    let map: std::collections::HashMap<i64, (f64, i64)> = rows
        .into_iter()
        .map(|r| (r.month, (r.revenue, r.invoice_count)))
        .collect();

    Ok((1i64..=12)
        .map(|m| {
            let (revenue, invoice_count) = map.get(&m).copied().unwrap_or((0.0, 0));
            MonthlyRevenue {
                month: m,
                month_name: MONTH_NAMES[(m - 1) as usize].to_string(),
                revenue,
                invoice_count,
            }
        })
        .collect())
}

async fn load_recent_invoices(
    db: &DatabaseConnection,
    year: i64,
) -> Result<Vec<crate::app::model::invoice::Invoice>, String> {
    #[derive(FromQueryResult)]
    struct IdRow {
        id: i64,
    }

    let rows = IdRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT id FROM invoices WHERE year=? AND status!='cancelled' ORDER BY issue_date DESC LIMIT 5",
        [year.into()],
    ))
    .all(db)
    .await
    .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        if let Ok(inv) = invoice_repository::load_invoice(db, row.id).await {
            results.push(inv);
        }
    }
    Ok(results)
}
