use std::collections::BTreeMap;

use sea_orm::{ActiveValue::Set, ConnectionTrait, DatabaseConnection, TransactionTrait};

use crate::app::entity::invoice as invoices;
use crate::app::entity::service as services;
use crate::app::model::appointment::Appointment as AppointmentModel;
use crate::app::model::invoice::{
    BulkUpdateStatusInput, CreateInvoiceInput, GenerateMonthlyInput, Invoice, InvoiceFilters,
    InvoiceLineInput, InvoiceStatus, MonthlyInvoicePreview, UpdateInvoiceInput,
};
use crate::app::model::tax::{InvoiceLineData, InvoiceTotals};
use crate::app::repository::invoice::invoice_repository;
use crate::app::repository::{appointment_repository, service_repository};
use crate::app::service::tax_service::{calculate_invoice_totals, rules_for};
use crate::app::service::ts::ts_submission_service;
use crate::app::service::validation_service as validate;

/// Lists invoices with optional filters (year, status, client_id, search).
pub async fn list(
    db: &DatabaseConnection,
    filters: InvoiceFilters,
) -> Result<Vec<Invoice>, String> {
    let ids = invoice_repository::find_ids(db, &filters).await?;
    invoice_repository::load_invoices(db, &ids).await
}

/// Returns a single invoice with its lines.
pub async fn get(db: &DatabaseConnection, id: i64) -> Result<Invoice, String> {
    invoice_repository::load_invoice(db, id).await
}

/// Creates a new invoice in a transaction and returns it.
pub async fn create(db: &DatabaseConnection, input: CreateInvoiceInput) -> Result<Invoice, String> {
    validate_invoice_input(
        input.client_id,
        &input.issue_date,
        input.due_date.as_deref(),
        &input.lines,
    )?;

    let tx = db.begin().await.map_err(|e| e.to_string())?;
    let id = create_in_tx(&tx, &input).await?;
    tx.commit().await.map_err(|e| e.to_string())?;

    invoice_repository::load_invoice(db, id).await
}

/// Updates an invoice in a transaction and returns the updated record.
///
/// The invoice number can be changed via `input.invoice_number` (manual
/// renumbering, e.g. to fill the gap left by a deleted invoice); the new
/// number must be unique within the invoice year.
pub async fn update(db: &DatabaseConnection, input: UpdateInvoiceInput) -> Result<Invoice, String> {
    validate::validate_id(input.id, "Fattura")?;
    validate_invoice_input(
        input.client_id,
        &input.issue_date,
        input.due_date.as_deref(),
        &input.lines,
    )?;

    let year = extract_year(&input.issue_date)?;
    let current = invoice_repository::load_invoice(db, input.id).await?;
    let number = resolve_invoice_number(input.invoice_number.as_deref(), &current.invoice_number)?;

    let tx = db.begin().await.map_err(|e| e.to_string())?;

    if invoice_repository::invoice_number_taken(&tx, year, number, input.id).await? {
        return Err(format!(
            "Numero fattura {number} già utilizzato nel {year}: scegli un numero libero"
        ));
    }

    let totals = compute_totals(&tx, &input.lines, input.apply_enpap).await?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let paid_date = resolve_paid_date(&input.status, input.paid_date.clone());
    let hide_quantity = input.hide_quantity.unwrap_or(current.hide_quantity);

    let active = invoices::ActiveModel {
        id: Set(input.id),
        client_id: Set(input.client_id),
        invoice_number: Set(format!("{number:03}")),
        year: Set(year),
        issue_date: Set(input.issue_date.clone()),
        due_date: Set(input.due_date.clone()),
        status: Set(input.status.as_str().to_owned()),
        payment_method: Set(input.payment_method.as_str().to_owned()),
        notes: Set(Some(input.notes.clone())),
        apply_enpap: Set(input.apply_enpap as i32),
        contributo_enpap: Set(totals.contributo_enpap),
        ritenuta_acconto: Set(totals.ritenuta_acconto),
        marca_da_bollo: Set((totals.marca_da_bollo > 0.0) as i32),
        total_net: Set(totals.total_net),
        total_tax: Set(totals.total_tax),
        total_gross: Set(totals.total_gross),
        total_due: Set(totals.total_due),
        paid_date: Set(paid_date),
        hide_quantity: Set(hide_quantity as i32),
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
    if ts_submission_service::blocks_invoice_deletion(db, id).await? {
        return Err(
            "Fattura trasmessa al Sistema TS: annulla la trasmissione prima di eliminarla"
                .to_string(),
        );
    }
    invoice_repository::delete_invoice(db, id)
        .await
        .map_err(|e| e.to_string())
}

/// Returns the next invoice number for the given year.
pub async fn next_number(db: &DatabaseConnection, year: i64) -> Result<String, String> {
    validate::validate_year(year)?;
    invoice_repository::next_invoice_number(db, year).await
}

/// Returns a preview of invoices that would be generated for the given month.
///
/// Groups completed, unbilled appointments by client, then by service.
pub async fn preview_monthly(
    db: &DatabaseConnection,
    year: i64,
    month: i64,
) -> Result<Vec<MonthlyInvoicePreview>, String> {
    validate::validate_year(year)?;
    validate::validate_month(month)?;

    let appointments = appointment_repository::find_unbilled_for_month(db, year, month).await?;
    if appointments.is_empty() {
        return Ok(vec![]);
    }

    let rules = rules_for(&invoice_repository::get_tax_profile(db).await?, true);

    let mut by_client: BTreeMap<i64, (String, Vec<_>)> = BTreeMap::new();
    for appt in &appointments {
        by_client
            .entry(appt.client_id)
            .or_insert_with(|| (appt.client_name.clone(), vec![]))
            .1
            .push(appt);
    }

    let svc_map = load_service_map(db).await?;

    let mut previews = Vec::new();
    for (client_id, (client_name, appts)) in &by_client {
        let lines = build_lines_from_appointments(appts, &svc_map);
        let line_data = to_line_data(&lines);
        let totals = calculate_invoice_totals(&line_data, &rules);

        previews.push(MonthlyInvoicePreview {
            client_id: *client_id,
            client_name: client_name.clone(),
            appointment_count: appts.len() as i64,
            lines,
            estimated_net: totals.total_net,
            estimated_due: totals.total_due,
        });
    }

    Ok(previews)
}

/// Generates invoices for the selected clients from their monthly appointments.
///
/// Each client's invoice and appointment links are committed atomically.
pub async fn generate_monthly(
    db: &DatabaseConnection,
    input: GenerateMonthlyInput,
) -> Result<Vec<Invoice>, String> {
    validate::validate_year(input.year)?;
    validate::validate_month(input.month)?;
    if input.client_ids.is_empty() {
        return Err("Seleziona almeno un cliente".to_string());
    }

    let appointments =
        appointment_repository::find_unbilled_for_month(db, input.year, input.month).await?;

    let mut by_client: BTreeMap<i64, Vec<_>> = BTreeMap::new();
    for appt in appointments {
        if input.client_ids.contains(&appt.client_id) {
            by_client.entry(appt.client_id).or_default().push(appt);
        }
    }

    let svc_map = load_service_map(db).await?;
    let issue_date = last_day_of_month(input.year, input.month)?;
    let mut created_ids = Vec::new();

    for (client_id, appts) in &by_client {
        let appt_ids: Vec<i64> = appts.iter().map(|a| a.id).collect();
        let appt_refs: Vec<&AppointmentModel> = appts.iter().collect();
        let lines = build_lines_from_appointments(&appt_refs, &svc_map);

        let invoice_input = CreateInvoiceInput {
            client_id: *client_id,
            issue_date: issue_date.clone(),
            due_date: None,
            status: InvoiceStatus::Issued,
            payment_method: input.payment_method.clone(),
            notes: String::new(),
            apply_enpap: input.apply_enpap,
            hide_quantity: None,
            lines,
        };

        let tx = db.begin().await.map_err(|e| e.to_string())?;
        let invoice_id = create_in_tx(&tx, &invoice_input).await?;
        appointment_repository::mark_as_invoiced(&tx, &appt_ids, invoice_id).await?;
        tx.commit().await.map_err(|e| e.to_string())?;

        created_ids.push(invoice_id);
    }

    invoice_repository::load_invoices(db, &created_ids).await
}

/// Updates the status of multiple invoices at once and returns the number
/// of rows actually updated.
///
/// When the target status is "paid", `paid_date` defaults to today if missing.
/// For any other status, `paid_date` is cleared.
pub async fn bulk_update_status(
    db: &DatabaseConnection,
    input: BulkUpdateStatusInput,
) -> Result<u64, String> {
    if input.ids.is_empty() {
        return Err("Nessuna fattura selezionata".to_string());
    }
    let paid_date = resolve_paid_date(&input.status, input.paid_date.clone());
    invoice_repository::bulk_update_status(db, &input.ids, input.status.as_str(), &paid_date).await
}

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Resolves the invoice number to store on update: the explicit override
/// when provided and non-blank, otherwise the number the invoice already has.
fn resolve_invoice_number(requested: Option<&str>, current: &str) -> Result<i64, String> {
    let value = requested
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(current);
    match value.parse::<i64>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(format!(
            "Numero fattura non valido: {value} (atteso un numero intero positivo)"
        )),
    }
}

/// Validates the shared fields of create/update invoice inputs.
fn validate_invoice_input(
    client_id: i64,
    issue_date: &str,
    due_date: Option<&str>,
    lines: &[InvoiceLineInput],
) -> Result<(), String> {
    validate::validate_id(client_id, "Cliente")?;
    validate::validate_invoice_dates(issue_date, due_date)?;
    validate::validate_invoice_lines(lines)?;
    Ok(())
}

/// Inserts an invoice with its lines inside an existing transaction.
async fn create_in_tx<C: ConnectionTrait>(
    tx: &C,
    input: &CreateInvoiceInput,
) -> Result<i64, String> {
    let year = extract_year(&input.issue_date)?;
    let invoice_number = invoice_repository::next_invoice_number(tx, year).await?;
    let totals = compute_totals(tx, &input.lines, input.apply_enpap).await?;
    let paid_date = resolve_paid_date(&input.status, None);
    let hide_quantity = match input.hide_quantity {
        Some(hide) => hide,
        None => invoice_repository::default_hide_quantity(tx, input.client_id).await?,
    };

    let active = invoices::ActiveModel {
        client_id: Set(input.client_id),
        invoice_number: Set(invoice_number),
        year: Set(year),
        issue_date: Set(input.issue_date.clone()),
        due_date: Set(input.due_date.clone()),
        status: Set(input.status.as_str().to_owned()),
        payment_method: Set(input.payment_method.as_str().to_owned()),
        notes: Set(Some(input.notes.clone())),
        apply_enpap: Set(input.apply_enpap as i32),
        contributo_enpap: Set(totals.contributo_enpap),
        ritenuta_acconto: Set(totals.ritenuta_acconto),
        marca_da_bollo: Set((totals.marca_da_bollo > 0.0) as i32),
        total_net: Set(totals.total_net),
        total_tax: Set(totals.total_tax),
        total_gross: Set(totals.total_gross),
        total_due: Set(totals.total_due),
        paid_date: Set(paid_date),
        hide_quantity: Set(hide_quantity as i32),
        ..Default::default()
    };

    let invoice = invoice_repository::insert_invoice(tx, active)
        .await
        .map_err(|e| e.to_string())?;
    invoice_repository::insert_lines(tx, invoice.id, &input.lines).await?;
    Ok(invoice.id)
}

/// Returns the paid date consistent with the target status: kept (or set to
/// today) for paid invoices, cleared for every other status.
fn resolve_paid_date(status: &InvoiceStatus, paid_date: Option<String>) -> Option<String> {
    if *status == InvoiceStatus::Paid {
        paid_date
            .filter(|d| !d.is_empty())
            .or_else(|| Some(chrono::Local::now().format("%Y-%m-%d").to_string()))
    } else {
        None
    }
}

/// Loads all services into a lookup map keyed by service id.
async fn load_service_map(
    db: &DatabaseConnection,
) -> Result<BTreeMap<i64, services::Model>, String> {
    let all = service_repository::find_all(db, false)
        .await
        .map_err(|e| e.to_string())?;
    Ok(all.into_iter().map(|s| (s.id, s)).collect())
}

/// Groups appointments by service and builds one invoice line per service type.
fn build_lines_from_appointments(
    appts: &[&AppointmentModel],
    svc_map: &BTreeMap<i64, services::Model>,
) -> Vec<InvoiceLineInput> {
    struct Group {
        service_name: String,
        price: f64,
        vat_rate: f64,
        service_id: Option<i64>,
        dates: Vec<String>,
    }

    let mut groups: BTreeMap<Option<i64>, Group> = BTreeMap::new();

    for a in appts {
        let (name, price, vat) = match a.service_id.and_then(|id| svc_map.get(&id)) {
            Some(svc) => (svc.name.as_str(), svc.default_price, svc.vat_rate),
            None => ("Seduta", 0.0, 0.0),
        };
        let entry = groups.entry(a.service_id).or_insert_with(|| Group {
            service_name: name.to_owned(),
            price,
            vat_rate: vat,
            service_id: a.service_id,
            dates: vec![],
        });
        entry.dates.push(format_date_short(&a.date));
    }

    groups
        .into_values()
        .map(|g| {
            let qty = g.dates.len() as i64;
            let dates_str = g.dates.join(", ");
            let description = if qty == 1 {
                format!("{} del {dates_str}", g.service_name)
            } else {
                format!("{} — {qty} sedute ({dates_str})", g.service_name)
            };
            InvoiceLineInput {
                service_id: g.service_id,
                description,
                quantity: qty,
                unit_price: g.price,
                vat_rate: g.vat_rate,
                amount_override: None,
            }
        })
        .collect()
}

fn to_line_data(lines: &[InvoiceLineInput]) -> Vec<InvoiceLineData> {
    lines.iter().map(InvoiceLineData::from).collect()
}

/// Returns the last day of the given month as an ISO date string.
///
/// Month must already be validated (1-12).
fn last_day_of_month(year: i64, month: i64) -> Result<String, String> {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    chrono::NaiveDate::from_ymd_opt(next_year as i32, next_month as u32, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.format("%Y-%m-%d").to_string())
        .ok_or_else(|| format!("Data non valida: {year}-{month}"))
}

fn format_date_short(iso: &str) -> String {
    if let Ok(d) = chrono::NaiveDate::parse_from_str(iso, "%Y-%m-%d") {
        d.format("%d/%m").to_string()
    } else {
        iso.to_owned()
    }
}

async fn compute_totals(
    db: &impl ConnectionTrait,
    lines: &[InvoiceLineInput],
    apply_enpap: bool,
) -> Result<InvoiceTotals, String> {
    let profile = invoice_repository::get_tax_profile(db).await?;
    Ok(calculate_invoice_totals(
        &to_line_data(lines),
        &rules_for(&profile, apply_enpap),
    ))
}

fn extract_year(date_str: &str) -> Result<i64, String> {
    use chrono::Datelike;
    let date = validate::parse_iso_date(date_str, "Data emissione")?;
    Ok(date.year() as i64)
}

#[cfg(test)]
#[path = "invoice_service_test.rs"]
mod tests;
