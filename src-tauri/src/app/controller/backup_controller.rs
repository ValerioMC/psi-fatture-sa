use std::path::PathBuf;

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::model::backup::{BackupFile, BackupOverview, BackupReason};
use crate::app::repository::backup_repository;
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

/// Where the archive and its automatic copies live, with the copies newest first.
#[tauri::command]
pub fn get_backup_overview(state: State<'_, AppState>) -> BackupOverview {
    state.backups.overview()
}

/// Adds a copy to the rotation now, dropping the oldest past the limit.
#[tauri::command]
pub async fn create_backup(state: State<'_, AppState>) -> Result<BackupFile, AppError> {
    let now = chrono::Local::now().naive_local();
    state
        .backups
        .back_up(&state.db, BackupReason::Manual, now)
        .await
}

/// Opens the backups folder in Finder or Esplora file, creating it if no copy exists yet.
#[tauri::command]
pub fn reveal_backups_folder(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    let folder = state.backups.folder();
    backup_repository::create_folder(folder)?;
    app.opener()
        .open_path(folder.to_string_lossy(), None::<&str>)
        .map_err(|e| AppError::External(format!("Cartella non apribile: {e}")))
}
