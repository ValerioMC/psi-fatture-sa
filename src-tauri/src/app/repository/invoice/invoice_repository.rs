use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, FromQueryResult, QueryFilter, QueryOrder, Statement,
};

use super::{InvoiceRow, NumberedDate};
use crate::app::common::AppError;
use crate::app::entity::{invoice as invoices, invoice_line};
use crate::app::model::config::TaxRegime;
use crate::app::model::invoice::{Invoice, InvoiceFilters, InvoiceLine, InvoiceLineInput};
use crate::app::model::tax::{round_cents, InvoiceLineData, TaxProfile};

/// Returns invoice ids matching the given filters.
pub async fn find_ids(
    db: &DatabaseConnection,
    filters: &InvoiceFilters,
) -> Result<Vec<i64>, AppError> {
    let mut conditions = vec!["1=1".to_string()];
    let mut values: Vec<sea_orm::Value> = Vec::new();

    if let Some(y) = filters.year {
        conditions.push("i.year = ?".to_string());
        values.push(y.into());
    }
    if let Some(s) = filters.status.as_deref().filter(|s| !s.is_empty()) {
        conditions.push("i.status = ?".to_string());
        values.push(s.into());
    }
    if let Some(cid) = filters.client_id {
        conditions.push("i.client_id = ?".to_string());
        values.push(cid.into());
    }
    if let Some(q) = filters
        .search
        .as_deref()
        .map(str::trim)
        .filter(|q| !q.is_empty())
    {
        let pattern = format!("%{q}%");
        conditions.push(
            "(lower(c.first_name || ' ' || c.last_name) LIKE lower(?) OR i.invoice_number LIKE ?)"
                .to_string(),
        );
        values.push(pattern.clone().into());
        values.push(pattern.into());
    }

    let sql = format!(
        "SELECT i.id FROM invoices i
         JOIN clients c ON i.client_id = c.id
         WHERE {} ORDER BY CAST(i.invoice_number AS INTEGER) DESC",
        conditions.join(" AND ")
    );

    #[derive(FromQueryResult)]
    struct IdRow {
        id: i64,
    }

    let rows = IdRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        &sql,
        values,
    ))
    .all(db)
    .await?;

    Ok(rows.into_iter().map(|r| r.id).collect())
}

/// Loads a full invoice (with client_name and lines) by id.
pub async fn load_invoice(db: &impl ConnectionTrait, id: i64) -> Result<Invoice, AppError> {
    let row = InvoiceRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT i.*, c.first_name || ' ' || c.last_name AS client_name
         FROM invoices i
         JOIN clients c ON i.client_id = c.id
         WHERE i.id = ?",
        [id.into()],
    ))
    .one(db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Fattura {id} non trovata")))?;

    let lines = load_lines(db, id).await?;
    Ok(row.into_invoice(lines))
}

/// Loads multiple invoices (with client_name and lines) in two queries,
/// preserving the order of the given ids.
pub async fn load_invoices(db: &DatabaseConnection, ids: &[i64]) -> Result<Vec<Invoice>, AppError> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let placeholders = vec!["?"; ids.len()].join(", ");
    let id_values: Vec<sea_orm::Value> = ids.iter().map(|id| (*id).into()).collect();

    let sql = format!(
        "SELECT i.*, c.first_name || ' ' || c.last_name AS client_name
         FROM invoices i
         JOIN clients c ON i.client_id = c.id
         WHERE i.id IN ({placeholders})"
    );
    let rows = InvoiceRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        &sql,
        id_values.clone(),
    ))
    .all(db)
    .await?;

    let lines_sql =
        format!("SELECT * FROM invoice_lines WHERE invoice_id IN ({placeholders}) ORDER BY id");
    let line_models = invoice_line::Model::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        &lines_sql,
        id_values,
    ))
    .all(db)
    .await?;

    let mut lines_by_invoice: std::collections::HashMap<i64, Vec<InvoiceLine>> =
        std::collections::HashMap::new();
    for m in line_models {
        lines_by_invoice
            .entry(m.invoice_id)
            .or_default()
            .push(into_line(m));
    }

    let mut by_id: std::collections::HashMap<i64, InvoiceRow> =
        rows.into_iter().map(|r| (r.id, r)).collect();

    Ok(ids
        .iter()
        .filter_map(|id| {
            by_id
                .remove(id)
                .map(|row| row.into_invoice(lines_by_invoice.remove(id).unwrap_or_default()))
        })
        .collect())
}

/// Inserts a new invoice record.
pub async fn insert_invoice(
    db: &impl sea_orm::ConnectionTrait,
    active: invoices::ActiveModel,
) -> Result<invoices::Model, sea_orm::DbErr> {
    active.insert(db).await
}

/// Updates an existing invoice record.
pub async fn update_invoice(
    db: &impl sea_orm::ConnectionTrait,
    active: invoices::ActiveModel,
) -> Result<invoices::Model, sea_orm::DbErr> {
    active.update(db).await
}

/// Deletes an invoice by id.
pub async fn delete_invoice(db: &DatabaseConnection, id: i64) -> Result<(), sea_orm::DbErr> {
    invoices::Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}

/// Inserts invoice lines for a given invoice id.
pub async fn insert_lines(
    db: &impl sea_orm::ConnectionTrait,
    invoice_id: i64,
    lines: &[InvoiceLineInput],
) -> Result<(), AppError> {
    for line in lines {
        let amounts = InvoiceLineData::from(line);

        let active = invoice_line::ActiveModel {
            invoice_id: Set(invoice_id),
            service_id: Set(line.service_id),
            description: Set(line.description.clone()),
            quantity: Set(line.quantity),
            unit_price: Set(line.unit_price),
            vat_rate: Set(line.vat_rate),
            line_total: Set(amounts.net_amount() + amounts.vat_amount()),
            amount_override: Set(line.amount_override.map(round_cents)),
            ..Default::default()
        };

        active.insert(db).await?;
    }
    Ok(())
}

/// Deletes all invoice lines for a given invoice id.
pub async fn delete_lines(
    db: &impl sea_orm::ConnectionTrait,
    invoice_id: i64,
) -> Result<(), sea_orm::DbErr> {
    invoice_line::Entity::delete_many()
        .filter(invoice_line::Column::InvoiceId.eq(invoice_id))
        .exec(db)
        .await?;
    Ok(())
}

/// Returns the next invoice number string for the given year.
///
/// Uses `initial_invoice_number` from config as a floor so numbering
/// never starts below the configured value.
pub async fn next_invoice_number(
    db: &impl sea_orm::ConnectionTrait,
    year: i64,
) -> Result<String, AppError> {
    #[derive(FromQueryResult)]
    struct Row {
        max_num: i64,
        initial: i64,
    }

    let row = Row::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT
           COALESCE(MAX(CAST(i.invoice_number AS INTEGER)), 0) AS max_num,
           COALESCE((SELECT initial_invoice_number FROM professional_config WHERE id = 1), 1) AS initial
         FROM invoices i WHERE i.year = ?",
        [year.into()],
    ))
    .one(db)
    .await?;

    let (max_num, initial) = row.map(|r| (r.max_num, r.initial)).unwrap_or((0, 1));
    let next = std::cmp::max(max_num, initial - 1) + 1;
    Ok(format!("{:03}", next))
}

/// Returns true when another invoice (different id) already uses the given
/// number in the given year. Numbers are compared as integers so "007" and
/// "7" count as the same number.
pub async fn invoice_number_taken(
    db: &impl sea_orm::ConnectionTrait,
    year: i64,
    number: i64,
    exclude_id: i64,
) -> Result<bool, AppError> {
    #[derive(FromQueryResult)]
    struct Row {
        n: i64,
    }

    let row = Row::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT COUNT(*) AS n FROM invoices
         WHERE year = ? AND CAST(invoice_number AS INTEGER) = ? AND id <> ?",
        [year.into(), number.into(), exclude_id.into()],
    ))
    .one(db)
    .await?;

    Ok(row.map(|r| r.n).unwrap_or(0) > 0)
}

/// The profile's tax settings; a missing profile taxes as forfettario with the bollo in the ENPAP base.
pub async fn get_tax_profile(db: &impl sea_orm::ConnectionTrait) -> Result<TaxProfile, AppError> {
    #[derive(FromQueryResult)]
    struct ProfileRow {
        tax_regime: String,
        enpap_excludes_bollo: i32,
    }

    let row = ProfileRow::find_by_statement(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT tax_regime, enpap_excludes_bollo FROM professional_config WHERE id = 1".to_owned(),
    ))
    .one(db)
    .await?;

    Ok(match row {
        Some(r) => TaxProfile {
            tax_regime: TaxRegime::from(r.tax_regime),
            enpap_excludes_bollo: r.enpap_excludes_bollo != 0,
        },
        None => TaxProfile {
            tax_regime: TaxRegime::Forfettario,
            enpap_excludes_bollo: false,
        },
    })
}

/// Whether a new invoice for this client hides quantity and unit price:
/// the client's own choice when it has one, otherwise the profile's.
pub async fn default_hide_quantity(
    db: &impl sea_orm::ConnectionTrait,
    client_id: i64,
) -> Result<bool, AppError> {
    #[derive(FromQueryResult)]
    struct HideRow {
        hide: i32,
    }

    let row = HideRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT COALESCE(
           (SELECT hide_quantity_in_invoice FROM clients WHERE id = ?),
           (SELECT hide_quantity_in_invoice FROM professional_config WHERE id = 1),
           0) AS hide",
        [client_id.into()],
    ))
    .one(db)
    .await?;

    Ok(row.is_some_and(|r| r.hide != 0))
}

/// Updates the status (and optionally paid_date) for multiple invoices in one
/// query, returning the number of rows actually updated.
pub async fn bulk_update_status(
    db: &DatabaseConnection,
    ids: &[i64],
    status: &str,
    paid_date: &Option<String>,
) -> Result<u64, AppError> {
    if ids.is_empty() {
        return Ok(0);
    }

    let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
    let in_clause = placeholders.join(", ");
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let sql = format!(
        "UPDATE invoices SET status = ?, paid_date = ?, updated_at = ? WHERE id IN ({in_clause})"
    );

    let mut values: Vec<sea_orm::Value> = vec![status.into(), paid_date.clone().into(), now.into()];
    for id in ids {
        values.push((*id).into());
    }

    let result = db
        .execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            &sql,
            values,
        ))
        .await?;

    Ok(result.rows_affected())
}

// ─── Private helpers ──────────────────────────────────────────────────────────

async fn load_lines(
    db: &impl ConnectionTrait,
    invoice_id: i64,
) -> Result<Vec<InvoiceLine>, AppError> {
    let models = invoice_line::Entity::find()
        .filter(invoice_line::Column::InvoiceId.eq(invoice_id))
        .order_by_asc(invoice_line::Column::Id)
        .all(db)
        .await?;

    Ok(models.into_iter().map(into_line).collect())
}

fn into_line(m: invoice_line::Model) -> InvoiceLine {
    InvoiceLine {
        id: Some(m.id),
        invoice_id: Some(m.invoice_id),
        service_id: m.service_id,
        description: m.description,
        quantity: m.quantity,
        unit_price: m.unit_price,
        vat_rate: m.vat_rate,
        line_total: m.line_total,
        amount_override: m.amount_override,
    }
}

/// The invoice issued on `issue_date` with `number`, if any: how a Sistema TS
/// document is traced back to its invoice.
pub async fn find_id_by_issue(
    db: &impl ConnectionTrait,
    issue_date: &str,
    number: &str,
) -> Result<Option<i64>, AppError> {
    #[derive(FromQueryResult)]
    struct IdRow {
        id: i64,
    }

    let row = IdRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT id FROM invoices WHERE issue_date = ? AND invoice_number = ?",
        [issue_date.into(), number.into()],
    ))
    .one(db)
    .await?;
    Ok(row.map(|r| r.id))
}

/// The first invoice of the year whose date breaks the numbering order around
/// `number` dated `issue_date`: a lower number dated later, or a higher one dated earlier.
pub async fn find_out_of_order(
    db: &impl ConnectionTrait,
    year: i64,
    number: i64,
    issue_date: &str,
    exclude_id: i64,
) -> Result<Option<NumberedDate>, AppError> {
    let row = NumberedDate::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT invoice_number, issue_date FROM invoices
         WHERE year = ? AND id <> ?
           AND ((CAST(invoice_number AS INTEGER) < ? AND issue_date > ?)
             OR (CAST(invoice_number AS INTEGER) > ? AND issue_date < ?))
         ORDER BY CAST(invoice_number AS INTEGER)
         LIMIT 1",
        [
            year.into(),
            exclude_id.into(),
            number.into(),
            issue_date.into(),
            number.into(),
            issue_date.into(),
        ],
    ))
    .one(db)
    .await?;
    Ok(row)
}
