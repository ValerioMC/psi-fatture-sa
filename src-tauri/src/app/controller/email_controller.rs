use tauri::State;

use crate::app::app_state::AppState;
use crate::app::common::AppError;
use crate::app::model::email::{
    EmailAccount, EmailConnectionCheck, EmailCredentialsStatus, EmailDraft, EmailPlaceholderInfo,
    EmailPreview, EmailProviderPreset, EmailTemplate, InvoiceEmail, InvoiceEmailFilters,
    SendInvoiceEmailInput, UpdateEmailAccountInput,
};
use crate::app::service::email::{
    email_account_service, email_credential_service, email_template_service, invoice_email_service,
};

#[tauri::command]
pub fn get_email_providers() -> Vec<EmailProviderPreset> {
    email_account_service::providers()
}

#[tauri::command]
pub async fn get_email_account(state: State<'_, AppState>) -> Result<EmailAccount, AppError> {
    email_account_service::get(&state.db).await
}

#[tauri::command]
pub async fn update_email_account(
    state: State<'_, AppState>,
    input: UpdateEmailAccountInput,
) -> Result<EmailAccount, AppError> {
    email_account_service::update(&state.db, input).await
}

/// Reports whether the mailbox password is stored; its value never leaves the backend.
#[tauri::command]
pub fn get_email_credentials_status(
    state: State<'_, AppState>,
) -> Result<EmailCredentialsStatus, AppError> {
    email_credential_service::status(state.secrets.as_ref())
}

#[tauri::command]
pub fn save_email_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<EmailCredentialsStatus, AppError> {
    email_credential_service::save_password(state.secrets.as_ref(), &password)
}

#[tauri::command]
pub fn delete_email_password(
    state: State<'_, AppState>,
) -> Result<EmailCredentialsStatus, AppError> {
    email_credential_service::delete_password(state.secrets.as_ref())
}

/// Logs in to the SMTP server once to tell whether the saved mailbox works.
#[tauri::command]
pub async fn check_email_connection(
    state: State<'_, AppState>,
) -> Result<EmailConnectionCheck, AppError> {
    invoice_email_service::check_connection(
        &state.db,
        state.secrets.as_ref(),
        state.mail_gateway.as_ref(),
    )
    .await
}

#[tauri::command]
pub async fn get_email_template(state: State<'_, AppState>) -> Result<EmailTemplate, AppError> {
    email_template_service::get(&state.db).await
}

#[tauri::command]
pub async fn update_email_template(
    state: State<'_, AppState>,
    template: EmailTemplate,
) -> Result<EmailTemplate, AppError> {
    email_template_service::update(&state.db, template).await
}

#[tauri::command]
pub async fn reset_email_template(state: State<'_, AppState>) -> Result<EmailTemplate, AppError> {
    email_template_service::reset(&state.db).await
}

#[tauri::command]
pub fn get_email_placeholders() -> Vec<EmailPlaceholderInfo> {
    email_template_service::placeholders()
}

/// The template as a patient would read it, filled with example values.
#[tauri::command]
pub async fn preview_email_template(
    state: State<'_, AppState>,
    template: EmailTemplate,
) -> Result<EmailPreview, AppError> {
    email_template_service::preview(&state.db, template).await
}

#[tauri::command]
pub async fn prepare_invoice_email(
    state: State<'_, AppState>,
    invoice_id: i64,
) -> Result<EmailDraft, AppError> {
    invoice_email_service::prepare(&state.db, invoice_id).await
}

#[tauri::command]
pub async fn send_invoice_email(
    state: State<'_, AppState>,
    input: SendInvoiceEmailInput,
) -> Result<InvoiceEmail, AppError> {
    invoice_email_service::send(
        &state.db,
        state.secrets.as_ref(),
        state.mail_gateway.as_ref(),
        input,
    )
    .await
}

/// Sends the template unchanged, for one invoice of a bulk send.
#[tauri::command]
pub async fn send_prepared_invoice_email(
    state: State<'_, AppState>,
    invoice_id: i64,
) -> Result<InvoiceEmail, AppError> {
    invoice_email_service::send_prepared(
        &state.db,
        state.secrets.as_ref(),
        state.mail_gateway.as_ref(),
        invoice_id,
    )
    .await
}

#[tauri::command]
pub async fn list_invoice_emails(
    state: State<'_, AppState>,
    filters: Option<InvoiceEmailFilters>,
) -> Result<Vec<InvoiceEmail>, AppError> {
    invoice_email_service::list(&state.db, filters.unwrap_or_default()).await
}
