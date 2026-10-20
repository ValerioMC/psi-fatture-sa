use tauri::State;

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::model::appointment::{
    Appointment, CreateAppointmentInput, CreateRecurringAppointmentsInput, UpdateAppointmentInput,
};
use crate::app::service::appointment_service;

#[tauri::command]
pub async fn list_appointments(
    state: State<'_, AppState>,
    date_from: Option<String>,
    date_to: Option<String>,
    client_id: Option<i64>,
) -> Result<Vec<Appointment>, AppError> {
    appointment_service::list(&state.db, date_from, date_to, client_id).await
}

#[tauri::command]
pub async fn create_appointment(
    state: State<'_, AppState>,
    input: CreateAppointmentInput,
) -> Result<Appointment, AppError> {
    appointment_service::create(&state.db, input).await
}

/// Creates multiple recurring appointments in a transaction.
#[tauri::command]
pub async fn create_recurring_appointments(
    state: State<'_, AppState>,
    input: CreateRecurringAppointmentsInput,
) -> Result<Vec<Appointment>, AppError> {
    appointment_service::create_recurring(&state.db, input).await
}

#[tauri::command]
pub async fn update_appointment(
    state: State<'_, AppState>,
    input: UpdateAppointmentInput,
) -> Result<Appointment, AppError> {
    appointment_service::update(&state.db, state.clock.as_ref(), input).await
}

#[tauri::command]
pub async fn delete_appointment(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    appointment_service::remove(&state.db, id).await
}
