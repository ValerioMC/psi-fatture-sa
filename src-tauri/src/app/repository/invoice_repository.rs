use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, FromQueryResult, QueryFilter, QueryOrder, Statement,
};

use crate::app::entity::{invoice as invoices, invoice_line};
use crate::app::model::invoice::{
    Invoice, InvoiceFilters, InvoiceLine, InvoiceLineInput, InvoiceStatus, PaymentMethod,
};

/// Returns invoice ids matching the given filters.
pub async fn find_ids(
    db: &DatabaseConnection,
    filters: &InvoiceFilters,
) -> Result<Vec<i64>, String> {
    let mut conditions = vec!["1=1".to_string()];

    if let Some(y) = filters.year {
        conditions.push(format!("i.year = {y}"));
    }
    if let Some(s) = &filters.status {
        if !s.is_empty() {
            let s = s.replace('\'', "''");
            conditions.push(format!("i.status = '{s}'"));
        }
    }
    if let Some(cid) = filters.client_id {
        conditions.push(format!("i.client_id = {cid}"));
    }
    if let Some(q) = &filters.search {
        if !q.trim().is_empty() {
            let q = q.trim().replace('\'', "''");
            conditions.push(format!(
                "(lower(c.first_name || ' ' || c.last_name) LIKE lower('%{q}%') OR i.invoice_number LIKE '%{q}%')"
            ));
        }
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

    let rows = IdRow::find_by_statement(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        sql,
    ))
    .all(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| r.id).collect())
}

/// Loads a full invoice (with client_name and lines) by id.
pub async fn load_invoice(db: &DatabaseConnection, id: i64) -> Result<Invoice, String> {
    #[derive(FromQueryResult)]
    struct InvoiceRow {
        id: i64,
        client_id: i64,
        client_name: String,
        invoice_number: String,
        year: i64,
        issue_date: String,
        due_date: Option<String>,
        status: String,
        payment_method: String,
        notes: Option<String>,
        apply_enpap: i32,
        contributo_enpap: f64,
        ritenuta_acconto: f64,
        marca_da_bollo: i32,
        total_net: f64,
        total_tax: f64,
        total_gross: f64,
        total_due: f64,
        paid_date: Option<String>,
        created_at: String,
        updated_at: String,
    }

    let row = InvoiceRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT i.*, c.first_name || ' ' || c.last_name AS client_name
         FROM invoices i
         JOIN clients c ON i.client_id = c.id
         WHERE i.id = ?",
        [id.into()],
    ))
    .one(db)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Invoice {id} not found"))?;

    let lines = load_lines(db, id).await?;

    Ok(Invoice {
        id: row.id,
        client_id: row.client_id,
        client_name: row.client_name,
        invoice_number: row.invoice_number,
        year: row.year,
        issue_date: row.issue_date,
        due_date: row.due_date,
        status: InvoiceStatus::from(row.status),
        payment_method: PaymentMethod::from(row.payment_method),
        notes: row.notes.unwrap_or_default(),
        apply_enpap: row.apply_enpap != 0,
        contributo_enpap: row.contributo_enpap,
        ritenuta_acconto: row.ritenuta_acconto,
        marca_da_bollo: row.marca_da_bollo != 0,
        total_net: row.total_net,
        total_tax: row.total_tax,
        total_gross: row.total_gross,
        total_due: row.total_due,
        paid_date: row.paid_date,
        lines,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
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
) -> Result<(), String> {
    for line in lines {
        let line_net = round2(line.quantity as f64 * line.unit_price);
        let line_vat = round2(line_net * line.vat_rate / 100.0);

        let active = invoice_line::ActiveModel {
            invoice_id: Set(invoice_id),
            service_id: Set(line.service_id),
            description: Set(line.description.clone()),
            quantity: Set(line.quantity),
            unit_price: Set(line.unit_price),
            vat_rate: Set(line.vat_rate),
            line_total: Set(line_net + line_vat),
            ..Default::default()
        };

        active.insert(db).await.map_err(|e| e.to_string())?;
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
) -> Result<String, String> {
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
    .await
    .map_err(|e| e.to_string())?;

    let (max_num, initial) = row.map(|r| (r.max_num, r.initial)).unwrap_or((0, 1));
    let next = std::cmp::max(max_num, initial - 1) + 1;
    Ok(format!("{:03}", next))
}

/// Returns the tax_regime from professional_config (defaults to "forfettario").
pub async fn get_tax_regime(db: &impl sea_orm::ConnectionTrait) -> Result<String, String> {
    #[derive(FromQueryResult)]
    struct RegimeRow {
        tax_regime: String,
    }

    let row = RegimeRow::find_by_statement(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT tax_regime FROM professional_config WHERE id = 1".to_owned(),
    ))
    .one(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row
        .map(|r| r.tax_regime)
        .unwrap_or_else(|| "forfettario".to_string()))
}

/// Updates the status (and optionally paid_date) for multiple invoices in one query.
pub async fn bulk_update_status(
    db: &DatabaseConnection,
    ids: &[i64],
    status: &str,
    paid_date: &Option<String>,
) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
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

    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        &sql,
        values,
    ))
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ─── Private helpers ──────────────────────────────────────────────────────────

async fn load_lines(db: &DatabaseConnection, invoice_id: i64) -> Result<Vec<InvoiceLine>, String> {
    let models = invoice_line::Entity::find()
        .filter(invoice_line::Column::InvoiceId.eq(invoice_id))
        .order_by_asc(invoice_line::Column::Id)
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(models
        .into_iter()
        .map(|m| InvoiceLine {
            id: Some(m.id),
            invoice_id: Some(m.invoice_id),
            service_id: m.service_id,
            description: m.description,
            quantity: m.quantity,
            unit_price: m.unit_price,
            vat_rate: m.vat_rate,
            line_total: m.line_total,
        })
        .collect())
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
