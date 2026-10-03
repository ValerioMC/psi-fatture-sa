use tauri::{AppHandle, State};

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::model::update::{Platform, UpdateCheck};
use crate::app::service::update_install_service::{self, UpdateTarget};
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

/// Installs the newest release over this app and restarts into it. On success it never
/// answers: the process is replaced. Mac only; elsewhere the download opens in the browser.
#[tauri::command]
pub async fn install_update(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    let current = app.package_info().version.to_string();
    let running_executable = std::env::current_exe()?;
    let work_dir = std::env::temp_dir().join("psi-fatture-update");
    update_install_service::install(UpdateTarget {
        gateway: state.release_gateway.as_ref(),
        installer: state.bundle_installer.as_ref(),
        current_version: &current,
        platform: Platform::current(),
        running_executable: &running_executable,
        work_dir: &work_dir,
    })
    .await?;
    app.restart()
}
