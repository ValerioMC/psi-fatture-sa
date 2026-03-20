use tauri::State;

use crate::app::model::client::{Client, CreateClientInput, UpdateClientInput};
use crate::app::service::client_service;
use crate::AppState;

/// Lists all clients, optionally filtered by search query.
#[tauri::command]
pub async fn list_clients(
    state: State<'_, AppState>,
    search: Option<String>,
) -> Result<Vec<Client>, String> {
    client_service::list(&state.db, search).await
}

/// Returns a single client by id.
#[tauri::command]
pub async fn get_client(state: State<'_, AppState>, id: i64) -> Result<Client, String> {
    client_service::get(&state.db, id).await
}

/// Creates a new client and returns it.
#[tauri::command]
pub async fn create_client(
    state: State<'_, AppState>,
    input: CreateClientInput,
) -> Result<Client, String> {
    client_service::create(&state.db, input).await
}

/// Updates an existing client and returns the updated record.
#[tauri::command]
pub async fn update_client(
    state: State<'_, AppState>,
    input: UpdateClientInput,
) -> Result<Client, String> {
    client_service::update(&state.db, input).await
}

/// Deletes a client. Fails if the client has associated invoices.
#[tauri::command]
pub async fn delete_client(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    client_service::remove(&state.db, id).await
}
