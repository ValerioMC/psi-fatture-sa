use std::path::PathBuf;

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::repository::document_file_repository;
use crate::app::service::pdf::invoice_pdf_service;

/// Renders the invoice PDF to a temporary file and opens it in the system viewer.
#[tauri::command]
pub async fn open_invoice_pdf(
    app: AppHandle,
    state: State<'_, AppState>,
    invoice_id: i64,
) -> Result<(), AppError> {
    let pdf = invoice_pdf_service::render(&state.db, invoice_id).await?;
    let path = document_file_repository::write_preview(&pdf.file_name, &pdf.bytes)?;
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
    let pdf = invoice_pdf_service::render(&state.db, invoice_id).await?;
    document_file_repository::write(&path, &pdf.bytes)
}

/// The name the save dialog proposes, "Fattura_12_2026.pdf".
#[tauri::command]
pub async fn get_invoice_pdf_name(
    state: State<'_, AppState>,
    invoice_id: i64,
) -> Result<String, AppError> {
    let document = invoice_pdf_service::load_document(&state.db, invoice_id).await?;
    Ok(invoice_pdf_service::file_name(&document.invoice))
}
