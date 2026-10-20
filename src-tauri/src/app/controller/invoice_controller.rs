use tauri::State;

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::model::invoice::{
    CreateInvoiceInput, GenerateMonthlyInput, Invoice, InvoiceFilters, InvoiceLineInput,
    MonthlyInvoicePreview,
};
use crate::app::model::tax::InvoiceTotals;
use crate::app::service::invoice::invoice_service;

/// The live totals of an invoice being edited, before it is saved.
#[tauri::command]
pub async fn preview_invoice_totals(
    state: State<'_, AppState>,
    lines: Vec<InvoiceLineInput>,
    apply_enpap: bool,
) -> Result<InvoiceTotals, AppError> {
    invoice_service::preview_totals(&state.db, &lines, apply_enpap).await
}

#[tauri::command]
pub async fn list_invoices(
    state: State<'_, AppState>,
    filters: InvoiceFilters,
) -> Result<Vec<Invoice>, AppError> {
    invoice_service::list(&state.db, filters).await
}

#[tauri::command]
pub async fn get_invoice(state: State<'_, AppState>, id: i64) -> Result<Invoice, AppError> {
    invoice_service::get(&state.db, id).await
}

#[tauri::command]
pub async fn create_invoice(
    state: State<'_, AppState>,
    input: CreateInvoiceInput,
) -> Result<Invoice, AppError> {
    invoice_service::create(&state.db, state.clock.as_ref(), input).await
}

#[tauri::command]
pub async fn get_next_invoice_number(
    state: State<'_, AppState>,
    year: i64,
) -> Result<String, AppError> {
    invoice_service::next_number(&state.db, year).await
}

#[tauri::command]
pub async fn preview_monthly_invoices(
    state: State<'_, AppState>,
    year: i64,
    month: i64,
) -> Result<Vec<MonthlyInvoicePreview>, AppError> {
    invoice_service::preview_monthly(&state.db, year, month).await
}

/// Creates invoices from completed appointments for selected clients.
#[tauri::command]
pub async fn generate_monthly_invoices(
    state: State<'_, AppState>,
    input: GenerateMonthlyInput,
) -> Result<Vec<Invoice>, AppError> {
    invoice_service::generate_monthly(&state.db, state.clock.as_ref(), input).await
}
