use tauri::State;

use crate::app::app_state::AppState;
use crate::app::model::terms::{AcceptTermsInput, TermsAcceptance};
use crate::app::service::terms_service;

/// The acceptance of that version of the terms of use, or None while it is pending.
#[tauri::command]
pub async fn get_terms_acceptance(
    state: State<'_, AppState>,
    version: String,
) -> Result<Option<TermsAcceptance>, String> {
    terms_service::find(&state.db, &version).await
}

#[tauri::command]
pub async fn accept_terms(
    state: State<'_, AppState>,
    input: AcceptTermsInput,
) -> Result<TermsAcceptance, String> {
    terms_service::accept(&state.db, input).await
}
