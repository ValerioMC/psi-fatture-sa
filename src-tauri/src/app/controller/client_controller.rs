use tauri::State;

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::model::client::{Client, CreateClientInput, UpdateClientInput};
use crate::app::service::client_service;

#[tauri::command]
pub async fn list_clients(
    state: State<'_, AppState>,
    search: Option<String>,
) -> Result<Vec<Client>, AppError> {
    client_service::list(&state.db, search).await
}

#[tauri::command]
pub async fn get_client(state: State<'_, AppState>, id: i64) -> Result<Client, AppError> {
    client_service::get(&state.db, id).await
}

#[tauri::command]
pub async fn create_client(
    state: State<'_, AppState>,
    input: CreateClientInput,
) -> Result<Client, AppError> {
    client_service::create(&state.db, input).await
}

#[tauri::command]
pub async fn update_client(
    state: State<'_, AppState>,
    input: UpdateClientInput,
) -> Result<Client, AppError> {
    client_service::update(&state.db, state.clock.as_ref(), input).await
}

/// Refused while the patient has invoices.
#[tauri::command]
pub async fn delete_client(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    client_service::remove(&state.db, id).await
}
