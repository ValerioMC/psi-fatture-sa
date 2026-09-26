/// Triggers the native OS print dialog for the current webview.
#[tauri::command]
pub fn print_current_page(webview: tauri::Webview) -> Result<(), String> {
    webview.print().map_err(|e| e.to_string())
}
