use sea_orm::{ActiveValue::Set, DatabaseConnection, TransactionTrait};

use crate::app::entity::invoice as invoices;
use crate::app::model::invoice::{
    CreateInvoiceInput, Invoice, InvoiceFilters, UpdateInvoiceInput,
};
use crate::app::repository::invoice_repository;
use crate::app::service::tax_service::{calculate_invoice_totals, InvoiceLineData};

/// Lists invoices with optional filters (year, status, client_id, search).
pub async fn list(
    db: &DatabaseConnection,
    filters: InvoiceFilters,
) -> Result<Vec<Invoice>, String> {
    let ids = invoice_repository::find_ids(db, &filters).await?;
    let mut results = Vec::with_capacity(ids.len());
    for id in ids {
        results.push(invoice_repository::load_invoice(db, id).await?);
    }
    Ok(results)
}

/// Returns a single invoice with its lines.
pub async fn get(db: &DatabaseConnection, id: i64) -> Result<Invoice, String> {
    invoice_repository::load_invoice(db, id).await
}

/// Creates a new invoice in a transaction and returns it.
pub async fn create(
    db: &DatabaseConnection,
    input: CreateInvoiceInput,
) -> Result<Invoice, String> {
    let tx = db.begin().await.map_err(|e| e.to_string())?;

    let year = extract_year(&input.issue_date)?;
    let invoice_number = invoice_repository::next_invoice_number(&tx, year).await?;
    let totals = compute_totals(&tx, &input.lines, input.apply_enpap).await?;

    let active = invoices::ActiveModel {
        client_id: Set(input.client_id),
        invoice_number: Set(invoice_number),
        year: Set(year),
        issue_date: Set(input.issue_date),
        due_date: Set(input.due_date),
        status: Set(input.status.as_str().to_owned()),
        payment_method: Set(input.payment_method.as_str().to_owned()),
        notes: Set(Some(input.notes)),
        apply_enpap: Set(input.apply_enpap as i32),
        contributo_enpap: Set(totals.contributo_enpap),
        ritenuta_acconto: Set(totals.ritenuta_acconto),
        marca_da_bollo: Set((totals.marca_da_bollo > 0.0) as i32),
        total_net: Set(totals.total_net),
        total_tax: Set(totals.total_tax),
        total_gross: Set(totals.total_gross),
        total_due: Set(totals.total_due),
        ..Default::default()
    };

    let invoice = invoice_repository::insert_invoice(&tx, active)
        .await
        .map_err(|e| e.to_string())?;
    invoice_repository::insert_lines(&tx, invoice.id, &input.lines).await?;
    tx.commit().await.map_err(|e| e.to_string())?;

    invoice_repository::load_invoice(db, invoice.id).await
}

/// Updates an invoice in a transaction and returns the updated record.
pub async fn update(
    db: &DatabaseConnection,
    input: UpdateInvoiceInput,
) -> Result<Invoice, String> {
    let tx = db.begin().await.map_err(|e| e.to_string())?;

    let totals = compute_totals(&tx, &input.lines, input.apply_enpap).await?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let active = invoices::ActiveModel {
        id: Set(input.id),
        client_id: Set(input.client_id),
        issue_date: Set(input.issue_date),
        due_date: Set(input.due_date),
        status: Set(input.status.as_str().to_owned()),
        payment_method: Set(input.payment_method.as_str().to_owned()),
        notes: Set(Some(input.notes)),
        apply_enpap: Set(input.apply_enpap as i32),
        contributo_enpap: Set(totals.contributo_enpap),
        ritenuta_acconto: Set(totals.ritenuta_acconto),
        marca_da_bollo: Set((totals.marca_da_bollo > 0.0) as i32),
        total_net: Set(totals.total_net),
        total_tax: Set(totals.total_tax),
        total_gross: Set(totals.total_gross),
        total_due: Set(totals.total_due),
        paid_date: Set(input.paid_date),
        updated_at: Set(now),
        ..Default::default()
    };

    invoice_repository::update_invoice(&tx, active)
        .await
        .map_err(|e| e.to_string())?;
    invoice_repository::delete_lines(&tx, input.id)
        .await
        .map_err(|e| e.to_string())?;
    invoice_repository::insert_lines(&tx, input.id, &input.lines).await?;
    tx.commit().await.map_err(|e| e.to_string())?;

    invoice_repository::load_invoice(db, input.id).await
}

/// Deletes an invoice by id.
pub async fn remove(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    invoice_repository::delete_invoice(db, id)
        .await
        .map_err(|e| e.to_string())
}

/// Returns the next invoice number for the given year.
pub async fn next_number(db: &DatabaseConnection, year: i64) -> Result<String, String> {
    invoice_repository::next_invoice_number(db, year).await
}

// ─── Private helpers ──────────────────────────────────────────────────────────

async fn compute_totals(
    db: &impl sea_orm::ConnectionTrait,
    lines: &[crate::app::model::invoice::InvoiceLineInput],
    apply_enpap: bool,
) -> Result<crate::app::service::tax_service::InvoiceTotals, String> {
    let regime = invoice_repository::get_tax_regime(db).await?;
    let enpap_rate = if apply_enpap { 2.0 } else { 0.0 };
    let ritenuta_rate = if regime == "ordinario" { 20.0 } else { 0.0 };

    let line_data: Vec<InvoiceLineData> = lines
        .iter()
        .map(|l| InvoiceLineData {
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
        })
        .collect();

    Ok(calculate_invoice_totals(&line_data, &regime, enpap_rate, ritenuta_rate))
}

fn extract_year(date_str: &str) -> Result<i64, String> {
    date_str
        .split('-')
        .next()
        .and_then(|y| y.parse::<i64>().ok())
        .ok_or_else(|| format!("Invalid date format: {date_str}"))
}
