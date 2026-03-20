mod commands;
mod db;
mod domain;

use commands::{
    appointments::{
        create_appointment, create_recurring_appointments, delete_appointment, get_appointment,
        list_appointments, update_appointment,
    },
    clients::{create_client, delete_client, get_client, list_clients, update_client},
    config::{get_config, upsert_config},
    dashboard::get_dashboard,
    invoices::{
        create_invoice, delete_invoice, get_invoice, get_next_invoice_number, list_invoices,
        update_invoice,
    },
    services::{create_service, delete_service, get_service, list_services, update_service},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // Config
            get_config,
            upsert_config,
            // Clients
            list_clients,
            get_client,
            create_client,
            update_client,
            delete_client,
            // Services
            list_services,
            get_service,
            create_service,
            update_service,
            delete_service,
            // Invoices
            list_invoices,
            get_invoice,
            create_invoice,
            update_invoice,
            delete_invoice,
            get_next_invoice_number,
            // Appointments
            list_appointments,
            get_appointment,
            create_appointment,
            create_recurring_appointments,
            update_appointment,
            delete_appointment,
            // Dashboard
            get_dashboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
