use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    FromQueryResult, QueryFilter, QueryOrder, Statement,
};

use crate::app::entity::{invoice_line, invoice as invoices};
use crate::app::model::invoice::{Invoice, InvoiceLine, InvoiceLineInput, InvoiceFilters, InvoiceStatus, PaymentMethod};

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
         WHERE {} ORDER BY i.issue_date DESC, i.invoice_number DESC",
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
pub async fn next_invoice_number(
    db: &impl sea_orm::ConnectionTrait,
    year: i64,
) -> Result<String, String> {
    #[derive(FromQueryResult)]
    struct MaxRow {
        max_num: i64,
    }

    let row = MaxRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT COALESCE(MAX(CAST(invoice_number AS INTEGER)), 0) AS max_num
         FROM invoices WHERE year = ?",
        [year.into()],
    ))
    .one(db)
    .await
    .map_err(|e| e.to_string())?;

    let max = row.map(|r| r.max_num).unwrap_or(0);
    Ok(format!("{:03}", max + 1))
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

    Ok(row.map(|r| r.tax_regime).unwrap_or_else(|| "forfettario".to_string()))
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
