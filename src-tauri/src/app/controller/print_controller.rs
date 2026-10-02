use crate::app::common::AppError;
/// Triggers the native OS print dialog for the current webview.
#[tauri::command]
pub fn print_current_page(webview: tauri::Webview) -> Result<(), AppError> {
    webview.print().map_err(AppError::from)
}
