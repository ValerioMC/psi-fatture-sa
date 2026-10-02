use tauri::State;

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::model::invoice::{
    BulkUpdateStatusInput, CreateInvoiceInput, GenerateMonthlyInput, Invoice, InvoiceFilters,
    InvoiceLineInput, MonthlyInvoicePreview, UpdateInvoiceInput,
};
use crate::app::model::tax::InvoiceTotals;
use crate::app::service::invoice_service;

/// The live totals of an invoice being edited, before it is saved.
#[tauri::command]
pub async fn preview_invoice_totals(
    state: State<'_, AppState>,
    lines: Vec<InvoiceLineInput>,
    apply_enpap: bool,
) -> Result<InvoiceTotals, AppError> {
    invoice_service::preview_totals(&state.db, &lines, apply_enpap).await
}

/// Lists invoices with optional filters (year, status, client_id, search).
#[tauri::command]
pub async fn list_invoices(
    state: State<'_, AppState>,
    filters: InvoiceFilters,
) -> Result<Vec<Invoice>, AppError> {
    invoice_service::list(&state.db, filters).await
}

/// Returns a single invoice with its lines.
#[tauri::command]
pub async fn get_invoice(state: State<'_, AppState>, id: i64) -> Result<Invoice, AppError> {
    invoice_service::get(&state.db, id).await
}

/// Creates a new invoice in a transaction and returns it.
#[tauri::command]
pub async fn create_invoice(
    state: State<'_, AppState>,
    input: CreateInvoiceInput,
) -> Result<Invoice, AppError> {
    invoice_service::create(&state.db, input).await
}

/// Updates an invoice in a transaction and returns the updated record.
#[tauri::command]
pub async fn update_invoice(
    state: State<'_, AppState>,
    input: UpdateInvoiceInput,
) -> Result<Invoice, AppError> {
    invoice_service::update(&state.db, input).await
}

/// Deletes an invoice by id.
#[tauri::command]
pub async fn delete_invoice(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    invoice_service::remove(&state.db, id).await
}

/// Returns the next invoice number for the given year.
#[tauri::command]
pub async fn get_next_invoice_number(
    state: State<'_, AppState>,
    year: i64,
) -> Result<String, AppError> {
    invoice_service::next_number(&state.db, year).await
}

/// Returns a preview of invoices that would be generated for the given month.
#[tauri::command]
pub async fn preview_monthly_invoices(
    state: State<'_, AppState>,
    year: i64,
    month: i64,
) -> Result<Vec<MonthlyInvoicePreview>, AppError> {
    invoice_service::preview_monthly(&state.db, year, month).await
}

/// Updates the status of multiple invoices in bulk.
#[tauri::command]
pub async fn bulk_update_invoice_status(
    state: State<'_, AppState>,
    input: BulkUpdateStatusInput,
) -> Result<u64, AppError> {
    invoice_service::bulk_update_status(&state.db, input).await
}

/// Creates invoices from completed appointments for selected clients.
#[tauri::command]
pub async fn generate_monthly_invoices(
    state: State<'_, AppState>,
    input: GenerateMonthlyInput,
) -> Result<Vec<Invoice>, AppError> {
    invoice_service::generate_monthly(&state.db, input).await
}
