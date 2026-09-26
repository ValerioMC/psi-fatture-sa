#![allow(clippy::module_inception)]

mod app;
mod migration;

use std::sync::Arc;

use sea_orm::DatabaseConnection;

use app::controller::{
    appointment_controller::*, client_controller::*, config_controller::*, dashboard_controller::*,
    invoice_controller::*, service_controller::*, ts_controller::*,
};
use app::repository::secret_store::{KeyringSecretStore, SecretStore};
use app::repository::sistema_ts::gateway::SistemaTsGateway;
use app::repository::sistema_ts::http_gateway::HttpSistemaTsGateway;

/// Application state shared across all Tauri commands.
pub struct AppState {
    pub db: DatabaseConnection,
    pub secrets: Arc<dyn SecretStore>,
    pub ts_gateway: Arc<dyn SistemaTsGateway>,
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

    let secrets: Arc<dyn SecretStore> = Arc::new(KeyringSecretStore);
    let ts_gateway: Arc<dyn SistemaTsGateway> =
        Arc::new(HttpSistemaTsGateway::new().expect("Failed to load the Sistema TS certificates"));
    app::controller::ts_worker::spawn(db.clone(), secrets.clone(), ts_gateway.clone());

    tauri::Builder::default()
        .manage(AppState {
            db,
            secrets,
            ts_gateway,
        })
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
            get_ts_credentials_status,
            save_ts_pincode,
            delete_ts_pincode,
            save_ts_password,
            delete_ts_password,
            get_ts_settings,
            update_ts_settings,
            check_ts_connection,
            list_ts_submissions,
            enqueue_ts_submission,
            enqueue_ts_replacement,
            enqueue_ts_cancellation,
            withdraw_ts_submission,
            dispatch_ts_queue,
            query_ts_invoice,
            get_ts_monthly_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
