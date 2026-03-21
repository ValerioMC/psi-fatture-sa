mod app;
mod migration;

use sea_orm::DatabaseConnection;

use app::controller::{
    appointment_controller::*,
    client_controller::*,
    config_controller::*,
    dashboard_controller::*,
    invoice_controller::*,
    service_controller::*,
};

/// Application state shared across all Tauri commands.
pub struct AppState {
    pub db: DatabaseConnection,
}

/// Triggers the native OS print dialog for the current webview.
#[tauri::command]
fn print_current_page(webview: tauri::Webview) -> Result<(), String> {
    webview.print().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = tauri::async_runtime::block_on(app::db::connection::init_db())
        .expect("Failed to initialize database");

    tauri::Builder::default()
        .manage(AppState { db })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            print_current_page,
            get_config,
            upsert_config,
            list_clients,
            get_client,
            create_client,
            update_client,
            delete_client,
            list_services,
            get_service,
            create_service,
            update_service,
            delete_service,
            list_invoices,
            get_invoice,
            create_invoice,
            update_invoice,
            delete_invoice,
            get_next_invoice_number,
            preview_monthly_invoices,
            generate_monthly_invoices,
            bulk_update_invoice_status,
            list_appointments,
            get_appointment,
            create_appointment,
            create_recurring_appointments,
            update_appointment,
            delete_appointment,
            get_dashboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
