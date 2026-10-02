#![allow(clippy::module_inception)]

mod app;
mod migration;
#[cfg(test)]
mod test_support;

use std::sync::Arc;

use app::app_state::AppState;
use app::controller::{
    appointment_controller::*, backup_controller::*, client_controller::*, config_controller::*,
    dashboard_controller::*, email_controller::*, invoice_controller::*, invoice_pdf_controller::*,
    print_controller::*, service_controller::*, terms_controller::*, ts::ts_controller::*,
};
use app::repository::email::{LettreMailGateway, MailGateway};
use app::repository::secret::{EncryptedFileSecretStore, OsMachineId, SecretStore};
use app::repository::ts::sistema_ts::{HttpSistemaTsGateway, SistemaTsGateway};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = tauri::async_runtime::block_on(app::db::connection::init_db())
        .expect("Failed to initialize database");

    let secrets: Arc<dyn SecretStore> = Arc::new(EncryptedFileSecretStore::new(
        app::db::connection::secrets_path(),
        Box::new(OsMachineId::default()),
    ));
    let ts_gateway: Arc<dyn SistemaTsGateway> =
        Arc::new(HttpSistemaTsGateway::new().expect("Failed to load the Sistema TS certificates"));
    app::scheduler::ts_worker::spawn(db.clone(), secrets.clone(), ts_gateway.clone());
    let mail_gateway: Arc<dyn MailGateway> = Arc::new(LettreMailGateway);

    tauri::Builder::default()
        .manage(AppState {
            db,
            secrets,
            ts_gateway,
            mail_gateway,
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
            get_ts_environments,
            get_ts_settings,
            update_ts_settings,
            check_ts_connection,
            list_ts_submissions,
            enqueue_ts_submission,
            enqueue_ts_replacement,
            export_backup,
            get_backup_file_name,
            preview_invoice_totals,
            is_ts_invoice_out_of_date,
            enqueue_ts_cancellation,
            withdraw_ts_submission,
            dispatch_ts_queue,
            query_ts_invoice,
            get_ts_monthly_report,
            get_email_providers,
            get_email_account,
            update_email_account,
            get_email_credentials_status,
            save_email_password,
            delete_email_password,
            check_email_connection,
            get_email_template,
            update_email_template,
            reset_email_template,
            get_email_placeholders,
            preview_email_template,
            prepare_invoice_email,
            send_invoice_email,
            send_prepared_invoice_email,
            list_invoice_emails,
            open_invoice_pdf,
            save_invoice_pdf,
            get_invoice_pdf_name,
            get_terms_acceptance,
            accept_terms,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
