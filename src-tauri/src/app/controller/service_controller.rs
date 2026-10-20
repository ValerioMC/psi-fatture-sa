use tauri::State;

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::model::service::{CreateServiceInput, Service, UpdateServiceInput};
use crate::app::service::service_service;

#[tauri::command]
pub async fn list_services(
    state: State<'_, AppState>,
    active_only: bool,
) -> Result<Vec<Service>, AppError> {
    service_service::list(&state.db, active_only).await
}

#[tauri::command]
pub async fn create_service(
    state: State<'_, AppState>,
    input: CreateServiceInput,
) -> Result<Service, AppError> {
    service_service::create(&state.db, state.clock.as_ref(), input).await
}

#[tauri::command]
pub async fn update_service(
    state: State<'_, AppState>,
    input: UpdateServiceInput,
) -> Result<Service, AppError> {
    service_service::update(&state.db, state.clock.as_ref(), input).await
}

#[tauri::command]
pub async fn delete_service(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    service_service::remove(&state.db, id).await
}
