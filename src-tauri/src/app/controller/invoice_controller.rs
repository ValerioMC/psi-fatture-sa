use tauri::State;

use crate::app::model::invoice::{
    CreateInvoiceInput, Invoice, InvoiceFilters, UpdateInvoiceInput,
};
use crate::app::service::invoice_service;
use crate::AppState;

/// Lists invoices with optional filters (year, status, client_id, search).
#[tauri::command]
pub async fn list_invoices(
    state: State<'_, AppState>,
    filters: InvoiceFilters,
) -> Result<Vec<Invoice>, String> {
    invoice_service::list(&state.db, filters).await
}

/// Returns a single invoice with its lines.
#[tauri::command]
pub async fn get_invoice(state: State<'_, AppState>, id: i64) -> Result<Invoice, String> {
    invoice_service::get(&state.db, id).await
}

/// Creates a new invoice in a transaction and returns it.
#[tauri::command]
pub async fn create_invoice(
    state: State<'_, AppState>,
    input: CreateInvoiceInput,
) -> Result<Invoice, String> {
    invoice_service::create(&state.db, input).await
}

/// Updates an invoice in a transaction and returns the updated record.
#[tauri::command]
pub async fn update_invoice(
    state: State<'_, AppState>,
    input: UpdateInvoiceInput,
) -> Result<Invoice, String> {
    invoice_service::update(&state.db, input).await
}

/// Deletes an invoice by id.
#[tauri::command]
pub async fn delete_invoice(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    invoice_service::remove(&state.db, id).await
}

/// Returns the next invoice number for the given year.
#[tauri::command]
pub async fn get_next_invoice_number(
    state: State<'_, AppState>,
    year: i64,
) -> Result<String, String> {
    invoice_service::next_number(&state.db, year).await
}
