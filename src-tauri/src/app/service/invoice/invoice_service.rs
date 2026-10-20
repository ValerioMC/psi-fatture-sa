use std::collections::BTreeMap;

use sea_orm::{ActiveValue::Set, ConnectionTrait, DatabaseConnection, TransactionTrait};

use super::{invoice_guard as guard, invoice_validation, ClientSessions, InvoiceContent};
use crate::app::common::{AppError, Clock};
use crate::app::entity::invoice as invoices;
use crate::app::entity::service as services;
use crate::app::model::invoice::invoice_number::format_invoice_number;
use crate::app::model::invoice::{
    BulkUpdateStatusInput, CreateInvoiceInput, GenerateMonthlyInput, Invoice, InvoiceFilters,
    InvoiceLineInput, InvoiceStatus, MonthlyInvoicePreview, UpdateInvoiceInput,
};
use crate::app::model::tax::tax_calculator::{calculate_invoice_totals, rules_for};
use crate::app::model::tax::{InvoiceLineData, InvoiceTotals, TaxRules};
use crate::app::repository::appointment::appointment_repository;
use crate::app::repository::invoice::invoice_repository;
use crate::app::repository::service_repository;
use crate::app::service::support::validation as validate;

/// The id a numbering check excludes for an invoice not saved yet: none.
const NEW_INVOICE_ID: i64 = 0;

pub async fn list(
    db: &DatabaseConnection,
    filters: InvoiceFilters,
) -> Result<Vec<Invoice>, AppError> {
    let ids = invoice_repository::find_ids(db, &filters).await?;
    invoice_repository::load_invoices(db, &ids).await
}

pub async fn get(db: &DatabaseConnection, id: i64) -> Result<Invoice, AppError> {
    invoice_repository::require(db, id).await
}

/// Saves the invoice with the year's next number.
pub async fn create(
    db: &DatabaseConnection,
    clock: &dyn Clock,
    input: CreateInvoiceInput,
) -> Result<Invoice, AppError> {
    validate_content(&InvoiceContent::from(&input))?;

    let tx = db.begin().await?;
    let id = create_in_tx(&tx, clock, &input).await?;
    tx.commit().await?;

    invoice_repository::require(db, id).await
}

/// Rewrites the invoice and its lines. `input.invoice_number` renumbers it, e.g. to
/// fill the gap a deleted invoice left, and must stay unique within the year.
pub async fn update(
    db: &DatabaseConnection,
    clock: &dyn Clock,
    input: UpdateInvoiceInput,
) -> Result<Invoice, AppError> {
    validate::validate_id(input.id, "Fattura")?;
    let content = InvoiceContent::from(&input);
    validate_content(&content)?;

    let year = extract_year(content.issue_date)?;
    let current = invoice_repository::require(db, input.id).await?;
    let number = requested_number(&input, &current)?;

    let tx = db.begin().await?;
    ensure_update_allowed(&tx, &current, &content, year, number).await?;

    let totals = compute_totals(&tx, content.lines, content.apply_enpap).await?;
    let hide_quantity = input.hide_quantity.unwrap_or(current.hide_quantity);
    let mut record = invoice_record(&content, year, number, &totals, hide_quantity, clock);
    record.id = Set(input.id);
    record.updated_at = Set(clock.timestamp());

    invoice_repository::update_invoice(&tx, record).await?;
    invoice_repository::delete_lines(&tx, input.id).await?;
    invoice_repository::insert_lines(&tx, input.id, content.lines).await?;
    tx.commit().await?;

    invoice_repository::require(db, input.id).await
}

pub async fn remove(db: &DatabaseConnection, id: i64) -> Result<(), AppError> {
    invoice_repository::delete_invoice(db, id)
        .await
        .map_err(AppError::from)
}

/// The totals the invoice would have with these lines under the saved tax profile:
/// the form's live summary, computed by the same code that saves the invoice.
pub async fn preview_totals(
    db: &DatabaseConnection,
    lines: &[InvoiceLineInput],
    apply_enpap: bool,
) -> Result<InvoiceTotals, AppError> {
    compute_totals(db, lines, apply_enpap).await
}

pub async fn next_number(db: &DatabaseConnection, year: i64) -> Result<String, AppError> {
    validate::validate_year(year)?;
    let number = invoice_repository::next_invoice_number(db, year).await?;
    Ok(format_invoice_number(number))
}

/// One preview per patient with completed, unbilled sessions in the month,
/// estimated with the ENPAP contribution applied.
pub async fn preview_monthly(
    db: &DatabaseConnection,
    year: i64,
    month: i64,
) -> Result<Vec<MonthlyInvoicePreview>, AppError> {
    validate::validate_year(year)?;
    validate::validate_month(month)?;

    let appointments = appointment_repository::find_unbilled_for_month(db, year, month).await?;
    if appointments.is_empty() {
        return Ok(vec![]);
    }

    let apply_enpap = true;
    let rules = rules_for(&invoice_repository::get_tax_profile(db).await?, apply_enpap);
    let services_by_id = load_services_by_id(db).await?;
    Ok(ClientSessions::group_by_client(appointments)
        .into_iter()
        .map(|client| monthly_preview(client, &services_by_id, &rules))
        .collect())
}

/// Invoices the month's unbilled sessions of the selected patients, one invoice each.
/// Nothing is created when any of them has a session without a price; past that,
/// each invoice is committed with its appointment links on its own.
pub async fn generate_monthly(
    db: &DatabaseConnection,
    clock: &dyn Clock,
    input: GenerateMonthlyInput,
) -> Result<Vec<Invoice>, AppError> {
    validate::validate_year(input.year)?;
    validate::validate_month(input.month)?;
    if input.client_ids.is_empty() {
        return Err(AppError::Invalid("Seleziona almeno un cliente".to_string()));
    }

    let issue_date = monthly_issue_date(&input)?;
    let services_by_id = load_services_by_id(db).await?;
    let billable: Vec<(ClientSessions, Vec<InvoiceLineInput>)> =
        selected_clients_sessions(db, &input)
            .await?
            .into_iter()
            .map(|client| {
                let lines = client.invoice_lines(&services_by_id);
                (client, lines)
            })
            .collect();
    ensure_every_session_priced(&billable)?;

    let mut created_ids = Vec::with_capacity(billable.len());
    for (client, lines) in billable {
        let id = create_monthly_invoice(db, clock, &input, &issue_date, &client, lines).await?;
        created_ids.push(id);
    }
    invoice_repository::load_invoices(db, &created_ids).await
}

/// Returns how many invoices changed status. "Paid" keeps the given paid date or
/// takes today; any other status clears it.
pub async fn bulk_update_status(
    db: &DatabaseConnection,
    clock: &dyn Clock,
    input: BulkUpdateStatusInput,
) -> Result<u64, AppError> {
    if input.ids.is_empty() {
        return Err(AppError::Invalid("Nessuna fattura selezionata".to_string()));
    }
    for id in &input.ids {
        validate::validate_id(*id, "Fattura")?;
    }
    let paid_date = resolve_paid_date(&input.status, input.paid_date.as_deref(), clock);
    let status = input.status.as_str();
    invoice_repository::bulk_update_status(db, &input.ids, status, &paid_date, &clock.timestamp())
        .await
}

/// The number an update gives the invoice: the one asked for, or the one it has.
pub fn requested_number(input: &UpdateInvoiceInput, current: &Invoice) -> Result<i64, AppError> {
    resolve_invoice_number(input.invoice_number.as_deref(), &current.invoice_number)
}

fn validate_content(content: &InvoiceContent) -> Result<(), AppError> {
    validate::validate_id(content.client_id, "Cliente")?;
    invoice_validation::validate_invoice_dates(content.issue_date, content.due_date)?;
    invoice_validation::validate_invoice_lines(content.lines)
}

async fn ensure_update_allowed(
    tx: &impl ConnectionTrait,
    current: &Invoice,
    content: &InvoiceContent<'_>,
    year: i64,
    number: i64,
) -> Result<(), AppError> {
    guard::ensure_number_free(tx, year, number, current.id).await?;
    guard::ensure_chronological(tx, year, number, content.issue_date, current.id).await
}

/// The explicit override when given and not blank, otherwise the number the invoice has.
fn resolve_invoice_number(requested: Option<&str>, current: &str) -> Result<i64, AppError> {
    let value = requested
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(current);
    match value.parse::<i64>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(AppError::Invalid(format!(
            "Numero fattura non valido: {value} (atteso un numero intero positivo)"
        ))),
    }
}

async fn create_in_tx<C: ConnectionTrait>(
    tx: &C,
    clock: &dyn Clock,
    input: &CreateInvoiceInput,
) -> Result<i64, AppError> {
    let content = InvoiceContent::from(input);
    let year = extract_year(content.issue_date)?;
    let number = invoice_repository::next_invoice_number(tx, year).await?;
    guard::ensure_chronological(tx, year, number, content.issue_date, NEW_INVOICE_ID).await?;

    let totals = compute_totals(tx, content.lines, content.apply_enpap).await?;
    let hide_quantity = match input.hide_quantity {
        Some(hide) => hide,
        None => invoice_repository::default_hide_quantity(tx, content.client_id).await?,
    };
    let record = invoice_record(&content, year, number, &totals, hide_quantity, clock);

    let invoice = invoice_repository::insert_invoice(tx, record).await?;
    invoice_repository::insert_lines(tx, invoice.id, content.lines).await?;
    Ok(invoice.id)
}

/// The invoice row as both creation and update store it; the caller adds the id it updates.
fn invoice_record(
    content: &InvoiceContent,
    year: i64,
    number: i64,
    totals: &InvoiceTotals,
    hide_quantity: bool,
    clock: &dyn Clock,
) -> invoices::ActiveModel {
    invoices::ActiveModel {
        client_id: Set(content.client_id),
        invoice_number: Set(format_invoice_number(number)),
        year: Set(year),
        issue_date: Set(content.issue_date.to_owned()),
        due_date: Set(content.due_date.map(str::to_owned)),
        status: Set(content.status.as_str().to_owned()),
        payment_method: Set(content.payment_method.as_str().to_owned()),
        notes: Set(Some(content.notes.to_owned())),
        apply_enpap: Set(content.apply_enpap as i32),
        contributo_enpap: Set(totals.contributo_enpap),
        ritenuta_acconto: Set(totals.ritenuta_acconto),
        marca_da_bollo: Set((totals.marca_da_bollo > 0.0) as i32),
        total_net: Set(totals.total_net),
        total_tax: Set(totals.total_tax),
        total_gross: Set(totals.total_gross),
        total_due: Set(totals.total_due),
        paid_date: Set(resolve_paid_date(content.status, content.paid_date, clock)),
        hide_quantity: Set(hide_quantity as i32),
        ..Default::default()
    }
}

/// Kept (or set to today) for paid invoices, cleared for every other status.
fn resolve_paid_date(
    status: &InvoiceStatus,
    paid_date: Option<&str>,
    clock: &dyn Clock,
) -> Option<String> {
    if *status != InvoiceStatus::Paid {
        return None;
    }
    Some(
        paid_date
            .filter(|date| !date.is_empty())
            .map_or_else(|| clock.today(), str::to_owned),
    )
}

async fn load_services_by_id(
    db: &DatabaseConnection,
) -> Result<BTreeMap<i64, services::Model>, AppError> {
    let all = service_repository::find_all(db, false).await?;
    Ok(all.into_iter().map(|s| (s.id, s)).collect())
}

fn monthly_preview(
    client: ClientSessions,
    services_by_id: &BTreeMap<i64, services::Model>,
    rules: &TaxRules,
) -> MonthlyInvoicePreview {
    let lines = client.invoice_lines(services_by_id);
    let totals = calculate_invoice_totals(&to_line_data(&lines), rules);
    MonthlyInvoicePreview {
        client_id: client.client_id,
        appointment_count: client.sessions.len() as i64,
        missing_price: lines.iter().any(line_has_no_price),
        client_name: client.client_name,
        lines,
        estimated_net: totals.total_net,
        estimated_due: totals.total_due,
    }
}

/// The chosen issue date, or the last day of the billed month when none is given.
fn monthly_issue_date(input: &GenerateMonthlyInput) -> Result<String, AppError> {
    match input.issue_date.as_deref().filter(|date| !date.is_empty()) {
        Some(date) => Ok(date.to_owned()),
        None => last_day_of_month(input.year, input.month),
    }
}

async fn selected_clients_sessions(
    db: &DatabaseConnection,
    input: &GenerateMonthlyInput,
) -> Result<Vec<ClientSessions>, AppError> {
    let selected = appointment_repository::find_unbilled_for_month(db, input.year, input.month)
        .await?
        .into_iter()
        .filter(|appointment| input.client_ids.contains(&appointment.client_id))
        .collect();
    Ok(ClientSessions::group_by_client(selected))
}

fn ensure_every_session_priced(
    billable: &[(ClientSessions, Vec<InvoiceLineInput>)],
) -> Result<(), AppError> {
    let unpriced: Vec<&str> = billable
        .iter()
        .filter(|(_, lines)| lines.iter().any(line_has_no_price))
        .map(|(client, _)| client.client_name.as_str())
        .collect();
    if unpriced.is_empty() {
        return Ok(());
    }
    Err(AppError::Invalid(format!(
        "Sedute senza prestazione o con prezzo zero per: {}. \
         Assegna una prestazione con un prezzo prima di fatturare",
        unpriced.join(", ")
    )))
}

async fn create_monthly_invoice(
    db: &DatabaseConnection,
    clock: &dyn Clock,
    input: &GenerateMonthlyInput,
    issue_date: &str,
    client: &ClientSessions,
    lines: Vec<InvoiceLineInput>,
) -> Result<i64, AppError> {
    let invoice_input = CreateInvoiceInput {
        client_id: client.client_id,
        issue_date: issue_date.to_owned(),
        due_date: None,
        status: InvoiceStatus::Issued,
        payment_method: input.payment_method.clone(),
        notes: String::new(),
        apply_enpap: input.apply_enpap,
        hide_quantity: None,
        lines,
    };

    let tx = db.begin().await?;
    let invoice_id = create_in_tx(&tx, clock, &invoice_input).await?;
    appointment_repository::mark_as_invoiced(&tx, &client.appointment_ids(), invoice_id).await?;
    tx.commit().await?;
    Ok(invoice_id)
}

/// A session whose service is missing or free would bill nothing.
fn line_has_no_price(line: &InvoiceLineInput) -> bool {
    InvoiceLineData::from(line).net_amount() <= 0.0
}

fn to_line_data(lines: &[InvoiceLineInput]) -> Vec<InvoiceLineData> {
    lines.iter().map(InvoiceLineData::from).collect()
}

/// Month must already be validated (1-12).
fn last_day_of_month(year: i64, month: i64) -> Result<String, AppError> {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    chrono::NaiveDate::from_ymd_opt(next_year as i32, next_month as u32, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.format("%Y-%m-%d").to_string())
        .ok_or_else(|| AppError::Invalid(format!("Data non valida: {year}-{month}")))
}

async fn compute_totals(
    db: &impl ConnectionTrait,
    lines: &[InvoiceLineInput],
    apply_enpap: bool,
) -> Result<InvoiceTotals, AppError> {
    let profile = invoice_repository::get_tax_profile(db).await?;
    Ok(calculate_invoice_totals(
        &to_line_data(lines),
        &rules_for(&profile, apply_enpap),
    ))
}

fn extract_year(date_str: &str) -> Result<i64, AppError> {
    use chrono::Datelike;
    let date = validate::parse_iso_date(date_str, "Data emissione")?;
    Ok(date.year() as i64)
}

#[cfg(test)]
#[path = "invoice_service_test.rs"]
mod tests;
