use tauri::State;

use crate::app::model::appointment::{
    Appointment, CreateAppointmentInput, CreateRecurringAppointmentsInput, UpdateAppointmentInput,
};
use crate::app::service::appointment_service;
use crate::AppState;

/// Lists appointments with optional date range and client filters.
#[tauri::command]
pub async fn list_appointments(
    state: State<'_, AppState>,
    date_from: Option<String>,
    date_to: Option<String>,
    client_id: Option<i64>,
) -> Result<Vec<Appointment>, String> {
    appointment_service::list(&state.db, date_from, date_to, client_id).await
}

/// Returns a single appointment by id.
#[tauri::command]
pub async fn get_appointment(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Appointment, String> {
    appointment_service::get(&state.db, id).await
}

/// Creates a single appointment and returns it.
#[tauri::command]
pub async fn create_appointment(
    state: State<'_, AppState>,
    input: CreateAppointmentInput,
) -> Result<Appointment, String> {
    appointment_service::create(&state.db, input).await
}

/// Creates multiple recurring appointments in a transaction.
#[tauri::command]
pub async fn create_recurring_appointments(
    state: State<'_, AppState>,
    input: CreateRecurringAppointmentsInput,
) -> Result<Vec<Appointment>, String> {
    appointment_service::create_recurring(&state.db, input).await
}

/// Updates an appointment and returns the updated record.
#[tauri::command]
pub async fn update_appointment(
    state: State<'_, AppState>,
    input: UpdateAppointmentInput,
) -> Result<Appointment, String> {
    appointment_service::update(&state.db, input).await
}

/// Deletes an appointment by id.
#[tauri::command]
pub async fn delete_appointment(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    appointment_service::remove(&state.db, id).await
}
