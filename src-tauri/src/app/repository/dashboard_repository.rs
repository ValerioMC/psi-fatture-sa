//! The yearly aggregates behind the dashboard. Cancelled invoices never count;
//! revenue is `total_due`, what the client actually pays.

use sea_orm::{DatabaseConnection, FromQueryResult, Statement};

use crate::app::model::dashboard::MonthTotal;

const RECENT_INVOICES_LIMIT: i64 = 5;

pub async fn revenue(db: &DatabaseConnection, year: i64) -> Result<f64, String> {
    query_f64(db, "SELECT CAST(COALESCE(SUM(total_due),0) AS REAL) AS val FROM invoices WHERE year=? AND status!='cancelled'", year).await
}

pub async fn net_revenue(db: &DatabaseConnection, year: i64) -> Result<f64, String> {
    query_f64(db, "SELECT CAST(COALESCE(SUM(total_net),0) AS REAL) AS val FROM invoices WHERE year=? AND status!='cancelled'", year).await
}

pub async fn paid_revenue(db: &DatabaseConnection, year: i64) -> Result<f64, String> {
    query_f64(db, "SELECT CAST(COALESCE(SUM(total_due),0) AS REAL) AS val FROM invoices WHERE year=? AND status='paid'", year).await
}

pub async fn unpaid_revenue(db: &DatabaseConnection, year: i64) -> Result<f64, String> {
    query_f64(db, "SELECT CAST(COALESCE(SUM(total_due),0) AS REAL) AS val FROM invoices WHERE year=? AND status IN ('issued','overdue')", year).await
}

pub async fn invoice_count(db: &DatabaseConnection, year: i64) -> Result<i64, String> {
    query_i64(
        db,
        "SELECT COUNT(*) AS val FROM invoices WHERE year=? AND status!='cancelled'",
        year,
    )
    .await
}

pub async fn paid_invoice_count(db: &DatabaseConnection, year: i64) -> Result<i64, String> {
    query_i64(
        db,
        "SELECT COUNT(*) AS val FROM invoices WHERE year=? AND status='paid'",
        year,
    )
    .await
}

pub async fn draft_invoice_count(db: &DatabaseConnection, year: i64) -> Result<i64, String> {
    query_i64(
        db,
        "SELECT COUNT(*) AS val FROM invoices WHERE year=? AND status='draft'",
        year,
    )
    .await
}

/// Paid revenue per month of `year`, by issue date; months without paid invoices are absent.
pub async fn paid_totals_by_month(
    db: &DatabaseConnection,
    year: i64,
) -> Result<Vec<MonthTotal>, String> {
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

    Ok(rows
        .into_iter()
        .map(|r| MonthTotal {
            month: r.month,
            revenue: r.revenue,
            invoice_count: r.invoice_count,
        })
        .collect())
}

/// The ids of the latest invoices issued in `year`, newest first.
pub async fn recent_invoice_ids(db: &DatabaseConnection, year: i64) -> Result<Vec<i64>, String> {
    #[derive(FromQueryResult)]
    struct IdRow {
        id: i64,
    }

    let rows = IdRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT id FROM invoices WHERE year=? AND status!='cancelled' ORDER BY issue_date DESC LIMIT ?",
        [year.into(), RECENT_INVOICES_LIMIT.into()],
    ))
    .all(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| r.id).collect())
}

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
