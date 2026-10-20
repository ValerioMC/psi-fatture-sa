use tauri::{AppHandle, State};

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::model::update::{Platform, UpdateCheck};
use crate::app::service::update::update_service;

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

/// Installs the newest release over this app and restarts into it. On success it never
/// answers: the process is replaced. Mac only; elsewhere the download opens in the browser.
#[tauri::command]
pub async fn install_update(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    let current = app.package_info().version.to_string();
    update_service::install_over_running_app(
        state.release_gateway.as_ref(),
        state.bundle_installer.as_ref(),
        &current,
        Platform::current(),
    )
    .await?;
    app.restart()
}
