use tauri::{AppHandle, State};

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::model::update::{Platform, UpdateCheck};
use crate::app::service::update_service;

/// The version of this build, as the release that produced it was tagged.
#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

/// Asks the release server whether a newer version is out, and where this system downloads it.
#[tauri::command]
pub async fn check_for_update(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<UpdateCheck, AppError> {
    let current = app.package_info().version.to_string();
    update_service::check(
        state.release_gateway.as_ref(),
        &current,
        Platform::current(),
    )
    .await
}
