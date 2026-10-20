use std::path::PathBuf;

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::service::pdf::invoice_pdf_service;

/// Renders the invoice PDF to a temporary file and opens it in the system viewer.
#[tauri::command]
pub async fn open_invoice_pdf(
    app: AppHandle,
    state: State<'_, AppState>,
    invoice_id: i64,
) -> Result<(), AppError> {
    let path = invoice_pdf_service::write_preview(&state.db, invoice_id).await?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|e| AppError::External(format!("PDF non apribile: {e}")))
}

/// Writes the invoice PDF where the professional chose to save it.
#[tauri::command]
pub async fn save_invoice_pdf(
    state: State<'_, AppState>,
    invoice_id: i64,
    path: PathBuf,
) -> Result<(), AppError> {
    invoice_pdf_service::save(&state.db, invoice_id, &path).await
}

/// The name the save dialog proposes, "Fattura_12_2026.pdf".
#[tauri::command]
pub async fn get_invoice_pdf_name(
    state: State<'_, AppState>,
    invoice_id: i64,
) -> Result<String, AppError> {
    invoice_pdf_service::file_name_of(&state.db, invoice_id).await
}

/// Opens the system print dialog on the page the window shows: the printable invoice.
#[tauri::command]
pub fn print_current_page(webview: tauri::Webview) -> Result<(), AppError> {
    webview.print().map_err(AppError::from)
}
