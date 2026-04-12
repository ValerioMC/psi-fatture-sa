use std::collections::BTreeMap;

use sea_orm::{ActiveValue::Set, DatabaseConnection, TransactionTrait};

use crate::app::entity::invoice as invoices;
use crate::app::entity::service as services;
use crate::app::model::appointment::Appointment as AppointmentModel;
use crate::app::model::invoice::{
    BulkUpdateStatusInput, CreateInvoiceInput, GenerateMonthlyInput, Invoice, InvoiceFilters,
    InvoiceLineInput, InvoiceStatus, MonthlyInvoicePreview, UpdateInvoiceInput,
};
use crate::app::repository::{appointment_repository, invoice_repository};
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
pub async fn create(db: &DatabaseConnection, input: CreateInvoiceInput) -> Result<Invoice, String> {
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
pub async fn update(db: &DatabaseConnection, input: UpdateInvoiceInput) -> Result<Invoice, String> {
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

/// Returns a preview of invoices that would be generated for the given month.
///
/// Groups completed, unbilled appointments by client, then by service.
pub async fn preview_monthly(
    db: &DatabaseConnection,
    year: i64,
    month: i64,
) -> Result<Vec<MonthlyInvoicePreview>, String> {
    let appointments = appointment_repository::find_unbilled_for_month(db, year, month).await?;
    if appointments.is_empty() {
        return Ok(vec![]);
    }

    let regime = invoice_repository::get_tax_regime(db).await?;

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
        let enpap_rate = 2.0;
        let ritenuta_rate = if regime == "ordinario" { 20.0 } else { 0.0 };
        let totals = calculate_invoice_totals(&line_data, &regime, enpap_rate, ritenuta_rate);

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
pub async fn generate_monthly(
    db: &DatabaseConnection,
    input: GenerateMonthlyInput,
) -> Result<Vec<Invoice>, String> {
    let appointments =
        appointment_repository::find_unbilled_for_month(db, input.year, input.month).await?;

    let mut by_client: BTreeMap<i64, Vec<_>> = BTreeMap::new();
    for appt in appointments {
        if input.client_ids.contains(&appt.client_id) {
            by_client.entry(appt.client_id).or_default().push(appt);
        }
    }

    let svc_map = load_service_map(db).await?;
    let issue_date = last_day_of_month(input.year, input.month);
    let mut created = Vec::new();

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
            lines,
        };

        let invoice = create(db, invoice_input).await?;
        appointment_repository::mark_as_invoiced(db, &appt_ids, invoice.id).await?;
        created.push(invoice);
    }

    Ok(created)
}

/// Updates the status of multiple invoices at once.
///
/// When the target status is "paid", sets `paid_date` to today.
/// For any other status, clears `paid_date`.
pub async fn bulk_update_status(
    db: &DatabaseConnection,
    input: BulkUpdateStatusInput,
) -> Result<u64, String> {
    let count = input.ids.len() as u64;
    invoice_repository::bulk_update_status(db, &input.ids, input.status.as_str(), &input.paid_date)
        .await?;
    Ok(count)
}

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Loads all services into a lookup map keyed by service id.
async fn load_service_map(
    db: &DatabaseConnection,
) -> Result<BTreeMap<i64, services::Model>, String> {
    use sea_orm::EntityTrait;
    let all = services::Entity::find()
        .all(db)
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
            }
        })
        .collect()
}

fn to_line_data(lines: &[InvoiceLineInput]) -> Vec<InvoiceLineData> {
    lines
        .iter()
        .map(|l| InvoiceLineData {
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
        })
        .collect()
}

fn last_day_of_month(year: i64, month: i64) -> String {
    let (ny, nm) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let next_first = chrono::NaiveDate::from_ymd_opt(ny as i32, nm as u32, 1)
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, 28).unwrap());
    let last = next_first.pred_opt().unwrap();
    last.format("%Y-%m-%d").to_string()
}

fn format_date_short(iso: &str) -> String {
    if let Ok(d) = chrono::NaiveDate::parse_from_str(iso, "%Y-%m-%d") {
        d.format("%d/%m").to_string()
    } else {
        iso.to_owned()
    }
}

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

    Ok(calculate_invoice_totals(
        &line_data,
        &regime,
        enpap_rate,
        ritenuta_rate,
    ))
}

fn extract_year(date_str: &str) -> Result<i64, String> {
    date_str
        .split('-')
        .next()
        .and_then(|y| y.parse::<i64>().ok())
        .ok_or_else(|| format!("Invalid date format: {date_str}"))
}
