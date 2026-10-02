use std::path::PathBuf;

use tauri::State;

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::service::backup_service;

/// Saves a copy of the whole archive where the professional chose.
#[tauri::command]
pub async fn export_backup(state: State<'_, AppState>, path: PathBuf) -> Result<(), AppError> {
    backup_service::export(&state.db, &path).await
}

/// The name the save dialog proposes for today's backup.
#[tauri::command]
pub fn get_backup_file_name() -> String {
    backup_service::file_name(chrono::Local::now().date_naive())
}
