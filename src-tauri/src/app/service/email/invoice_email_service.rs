use sea_orm::{ActiveValue::Set, ConnectionTrait};

use super::{email_account_service, email_credential_service, email_template_service, EmailValues};
use crate::app::entity::invoice_email::{self, ActiveModel};
use crate::app::model::email::{
    EmailAccount, EmailConnectionCheck, EmailDraft, InvoiceEmail, InvoiceEmailFilters,
    InvoiceEmailStatus, SendInvoiceEmailInput,
};
use crate::app::model::invoice::InvoiceStatus;
use crate::app::repository::client_repository;
use crate::app::repository::email::{
    invoice_email_repository, MailAttachment, MailGateway, OutgoingMail,
};
use crate::app::repository::secret::SecretStore;
use crate::app::service::pdf::{invoice_pdf_service, InvoiceDocument};
use crate::app::service::validation_service as validate;

/// The email the template proposes for this invoice, addressed to the patient's
/// address on file when there is one.
pub async fn prepare(db: &impl ConnectionTrait, invoice_id: i64) -> Result<EmailDraft, String> {
    let document = invoice_pdf_service::load_document(db, invoice_id).await?;
    ensure_sendable(&document)?;
    let template = email_template_service::get(db).await?;
    let filled = email_template_service::fill(&template, &EmailValues::of(&document));
    let recipient = document
        .client
        .email
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .to_string();
    Ok(EmailDraft {
        invoice_id,
        recipient_on_file: !recipient.is_empty(),
        recipient,
        subject: filled.subject,
        body: filled.body,
        attachment_name: invoice_pdf_service::file_name(&document.invoice),
    })
}

/// Sends the email with the invoice PDF attached and records the attempt either way.
/// A failure comes back as the error, after being recorded.
pub async fn send(
    db: &impl ConnectionTrait,
    secrets: &dyn SecretStore,
    gateway: &dyn MailGateway,
    input: SendInvoiceEmailInput,
) -> Result<InvoiceEmail, String> {
    let recipient = input.recipient.trim().to_lowercase();
    validate::validate_required(&recipient, "Destinatario")?;
    validate::validate_email(&recipient)?;
    let subject = input.subject.trim().to_string();
    validate::validate_required(&subject, "Oggetto")?;
    validate::validate_required(&input.body, "Testo dell'email")?;

    let account = email_account_service::saved(db).await?;
    let session = email_credential_service::session(secrets, &account)?;
    let document = invoice_pdf_service::load_document(db, input.invoice_id).await?;
    ensure_sendable(&document)?;
    let pdf = invoice_pdf_service::render_document(&document)?;

    let mail = OutgoingMail {
        from_address: account.sender_address.clone(),
        from_name: account.sender_name.clone(),
        to: recipient.clone(),
        bcc: bcc(&account, &recipient),
        subject: subject.clone(),
        body: input.body.trim_end().to_string(),
        attachment: Some(MailAttachment {
            file_name: pdf.file_name.clone(),
            content_type: "application/pdf".to_string(),
            bytes: pdf.bytes,
        }),
    };
    let outcome = gateway.send(&session, &mail).await;
    let error = outcome.as_ref().err().map(ToString::to_string);
    let record = record(
        db,
        input.invoice_id,
        &recipient,
        &subject,
        &pdf.file_name,
        error.clone(),
    )
    .await?;
    if let Some(error) = error {
        log::warn!(target: "email", invoice_id = input.invoice_id; "invoice email not sent");
        return Err(error);
    }
    if input.remember_recipient
        && document
            .client
            .email
            .as_deref()
            .is_none_or(|e| e.trim().is_empty())
    {
        client_repository::update_email(db, document.client.id, &recipient)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(record)
}

/// Sends the template as it is, for invoices emailed in bulk from the list.
pub async fn send_prepared(
    db: &impl ConnectionTrait,
    secrets: &dyn SecretStore,
    gateway: &dyn MailGateway,
    invoice_id: i64,
) -> Result<InvoiceEmail, String> {
    let draft = prepare(db, invoice_id).await?;
    if draft.recipient.is_empty() {
        return Err("Il paziente non ha un indirizzo email nella sua scheda".to_string());
    }
    send(
        db,
        secrets,
        gateway,
        SendInvoiceEmailInput {
            invoice_id,
            recipient: draft.recipient,
            subject: draft.subject,
            body: draft.body,
            remember_recipient: false,
        },
    )
    .await
}

pub async fn list(
    db: &impl ConnectionTrait,
    filters: InvoiceEmailFilters,
) -> Result<Vec<InvoiceEmail>, String> {
    invoice_email_repository::find(db, &filters)
        .await?
        .into_iter()
        .map(into_domain)
        .collect()
}

/// Logs in to the SMTP server once, sending nothing.
pub async fn check_connection(
    db: &impl ConnectionTrait,
    secrets: &dyn SecretStore,
    gateway: &dyn MailGateway,
) -> Result<EmailConnectionCheck, String> {
    let account = email_account_service::saved(db).await?;
    let session = email_credential_service::session(secrets, &account)?;
    Ok(match gateway.check(&session).await {
        Ok(()) => EmailConnectionCheck {
            ok: true,
            message: format!(
                "Collegamento riuscito: le fatture partiranno da {}",
                account.sender_address
            ),
        },
        Err(error) => EmailConnectionCheck {
            ok: false,
            message: error.to_string(),
        },
    })
}

/// A draft has no fiscal value yet and a cancelled invoice has none any more.
fn ensure_sendable(document: &InvoiceDocument) -> Result<(), String> {
    match document.invoice.status {
        InvoiceStatus::Draft => Err("Emetti la fattura prima di inviarla al paziente".to_string()),
        InvoiceStatus::Cancelled => {
            Err("Una fattura annullata non si invia al paziente".to_string())
        }
        InvoiceStatus::Issued | InvoiceStatus::Paid | InvoiceStatus::Overdue => Ok(()),
    }
}

/// A copy to the sender, unless the sender is the recipient.
fn bcc(account: &EmailAccount, recipient: &str) -> Option<String> {
    (account.bcc_self && account.sender_address != recipient)
        .then(|| account.sender_address.clone())
}

async fn record(
    db: &impl ConnectionTrait,
    invoice_id: i64,
    recipient: &str,
    subject: &str,
    attachment_name: &str,
    error: Option<String>,
) -> Result<InvoiceEmail, String> {
    let status = if error.is_some() {
        InvoiceEmailStatus::Failed
    } else {
        InvoiceEmailStatus::Sent
    };
    let active = ActiveModel {
        invoice_id: Set(invoice_id),
        recipient: Set(recipient.to_string()),
        subject: Set(subject.to_string()),
        attachment_name: Set(attachment_name.to_string()),
        status: Set(status.as_str().to_string()),
        error: Set(error),
        sent_at: Set(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        ..Default::default()
    };
    into_domain(invoice_email_repository::insert(db, active).await?)
}

fn into_domain(row: invoice_email::Model) -> Result<InvoiceEmail, String> {
    Ok(InvoiceEmail {
        id: row.id,
        invoice_id: row.invoice_id,
        recipient: row.recipient,
        subject: row.subject,
        attachment_name: row.attachment_name,
        status: InvoiceEmailStatus::parse(&row.status)?,
        error: row.error,
        sent_at: row.sent_at,
    })
}

#[cfg(test)]
#[path = "invoice_email_service_test.rs"]
mod tests;
