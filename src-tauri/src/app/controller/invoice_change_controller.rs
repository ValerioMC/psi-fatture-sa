//! The invoice commands the Sistema TS can refuse: they go through its rules first.

use tauri::State;

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::model::invoice::{BulkUpdateStatusInput, Invoice, UpdateInvoiceInput};
use crate::app::service::invoice_change_service;

#[tauri::command]
pub async fn update_invoice(
    state: State<'_, AppState>,
    input: UpdateInvoiceInput,
) -> Result<Invoice, AppError> {
    invoice_change_service::update(&state.db, state.clock.as_ref(), input).await
}

#[tauri::command]
pub async fn delete_invoice(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    invoice_change_service::remove(&state.db, id).await
}

#[tauri::command]
pub async fn bulk_update_invoice_status(
    state: State<'_, AppState>,
    input: BulkUpdateStatusInput,
) -> Result<u64, AppError> {
    invoice_change_service::bulk_update_status(&state.db, state.clock.as_ref(), input).await
}
