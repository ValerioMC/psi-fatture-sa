use tauri::State;

use crate::app::model::config::{ProfessionalConfig, UpsertConfigInput};
use crate::app::service::config_service;
use crate::AppState;

/// Returns the professional config, or None if not yet configured.
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Option<ProfessionalConfig>, String> {
    config_service::get(&state.db).await
}

/// Inserts or updates the professional config (singleton, id=1).
#[tauri::command]
pub async fn upsert_config(
    state: State<'_, AppState>,
    input: UpsertConfigInput,
) -> Result<ProfessionalConfig, String> {
    config_service::upsert(&state.db, input).await
}
