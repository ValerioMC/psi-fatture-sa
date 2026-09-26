use tauri::State;

use crate::app::model::ts::{
    TsConnectionCheck, TsCredentialsStatus, TsDispatchSummary, TsQueryResult, TsReportBasis,
    TsReportRow, TsSettings, TsSubmission, TsSubmissionFilters, UpdateTsSettingsInput,
};
use crate::app::service::{
    ts_credential_service, ts_dispatch_service, ts_remote_service, ts_settings_service,
    ts_submission_service,
};
use crate::AppState;

/// Reports which secrets are stored; their values never leave the backend.
#[tauri::command]
pub fn get_ts_credentials_status(
    state: State<'_, AppState>,
) -> Result<TsCredentialsStatus, String> {
    ts_credential_service::status(state.secrets.as_ref())
}

#[tauri::command]
pub fn save_ts_pincode(
    state: State<'_, AppState>,
    pincode: String,
) -> Result<TsCredentialsStatus, String> {
    ts_credential_service::save_pincode(state.secrets.as_ref(), &pincode)
}

#[tauri::command]
pub fn delete_ts_pincode(state: State<'_, AppState>) -> Result<TsCredentialsStatus, String> {
    ts_credential_service::delete_pincode(state.secrets.as_ref())
}

#[tauri::command]
pub fn save_ts_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<TsCredentialsStatus, String> {
    ts_credential_service::save_password(state.secrets.as_ref(), &password)
}

#[tauri::command]
pub fn delete_ts_password(state: State<'_, AppState>) -> Result<TsCredentialsStatus, String> {
    ts_credential_service::delete_password(state.secrets.as_ref())
}

#[tauri::command]
pub async fn get_ts_settings(state: State<'_, AppState>) -> Result<TsSettings, String> {
    ts_settings_service::get(&state.db).await
}

#[tauri::command]
pub async fn update_ts_settings(
    state: State<'_, AppState>,
    input: UpdateTsSettingsInput,
) -> Result<TsSettings, String> {
    ts_settings_service::update(&state.db, input).await
}

/// Calls the Sistema TS once to tell whether the stored credentials work.
#[tauri::command]
pub async fn check_ts_connection(state: State<'_, AppState>) -> Result<TsConnectionCheck, String> {
    ts_remote_service::check_connection(
        &state.db,
        state.secrets.as_ref(),
        state.ts_gateway.as_ref(),
    )
    .await
}

#[tauri::command]
pub async fn list_ts_submissions(
    state: State<'_, AppState>,
    filters: Option<TsSubmissionFilters>,
) -> Result<Vec<TsSubmission>, String> {
    ts_submission_service::list(&state.db, filters.unwrap_or_default()).await
}

#[tauri::command]
pub async fn enqueue_ts_submission(
    state: State<'_, AppState>,
    invoice_id: i64,
) -> Result<TsSubmission, String> {
    ts_submission_service::enqueue_invio(&state.db, invoice_id).await
}

#[tauri::command]
pub async fn enqueue_ts_replacement(
    state: State<'_, AppState>,
    submission_id: i64,
) -> Result<TsSubmission, String> {
    ts_submission_service::enqueue_replacement(&state.db, submission_id).await
}

#[tauri::command]
pub async fn enqueue_ts_cancellation(
    state: State<'_, AppState>,
    submission_id: i64,
) -> Result<TsSubmission, String> {
    ts_submission_service::enqueue_cancellation(&state.db, submission_id).await
}

/// Removes a submission that has not been sent yet.
#[tauri::command]
pub async fn withdraw_ts_submission(
    state: State<'_, AppState>,
    submission_id: i64,
) -> Result<(), String> {
    ts_submission_service::withdraw(&state.db, submission_id).await
}

/// Sends what is due now instead of waiting for the background worker.
#[tauri::command]
pub async fn dispatch_ts_queue(state: State<'_, AppState>) -> Result<TsDispatchSummary, String> {
    ts_dispatch_service::dispatch_due(
        &state.db,
        state.secrets.as_ref(),
        state.ts_gateway.as_ref(),
        chrono::Utc::now().naive_utc(),
    )
    .await
}

#[tauri::command]
pub async fn query_ts_invoice(
    state: State<'_, AppState>,
    invoice_id: i64,
) -> Result<TsQueryResult, String> {
    ts_remote_service::query_invoice(
        &state.db,
        state.secrets.as_ref(),
        state.ts_gateway.as_ref(),
        invoice_id,
    )
    .await
}

#[tauri::command]
pub async fn get_ts_monthly_report(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
    basis: TsReportBasis,
) -> Result<Vec<TsReportRow>, String> {
    ts_remote_service::monthly_report(
        &state.db,
        state.secrets.as_ref(),
        state.ts_gateway.as_ref(),
        year,
        month,
        basis,
    )
    .await
}
