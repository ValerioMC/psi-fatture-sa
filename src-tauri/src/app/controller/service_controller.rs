use tauri::State;

use crate::app::model::service::{CreateServiceInput, Service, UpdateServiceInput};
use crate::app::service::service_service;
use crate::AppState;

/// Lists services, optionally filtering to active-only.
#[tauri::command]
pub async fn list_services(
    state: State<'_, AppState>,
    active_only: bool,
) -> Result<Vec<Service>, String> {
    service_service::list(&state.db, active_only).await
}

/// Returns a single service by id.
#[tauri::command]
pub async fn get_service(state: State<'_, AppState>, id: i64) -> Result<Service, String> {
    service_service::get(&state.db, id).await
}

/// Creates a new service and returns it.
#[tauri::command]
pub async fn create_service(
    state: State<'_, AppState>,
    input: CreateServiceInput,
) -> Result<Service, String> {
    service_service::create(&state.db, input).await
}

/// Updates an existing service and returns the updated record.
#[tauri::command]
pub async fn update_service(
    state: State<'_, AppState>,
    input: UpdateServiceInput,
) -> Result<Service, String> {
    service_service::update(&state.db, input).await
}

/// Deletes a service by id.
#[tauri::command]
pub async fn delete_service(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    service_service::remove(&state.db, id).await
}
