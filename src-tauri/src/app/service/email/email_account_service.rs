use sea_orm::{ActiveValue::Set, ConnectionTrait};

use crate::app::entity::email_account::{self, ActiveModel};
use crate::app::model::email::{
    EmailAccount, EmailProvider, EmailProviderPreset, EmailSecurity, UpdateEmailAccountInput,
};
use crate::app::repository::config_repository;
use crate::app::repository::email::email_account_repository;
use crate::app::service::validation_service as validate;

const NAME_MAX_LENGTH: usize = 100;

pub fn providers() -> Vec<EmailProviderPreset> {
    EmailProvider::ALL
        .iter()
        .map(EmailProvider::preset)
        .collect()
}

/// The saved mailbox, or one proposed from the profile's PEC address before the first save.
pub async fn get(db: &impl ConnectionTrait) -> Result<EmailAccount, String> {
    match email_account_repository::find(db).await? {
        Some(row) => from_row(row),
        None => proposed(db).await,
    }
}

/// The mailbox to send from, refused until the professional has saved one.
pub async fn saved(db: &impl ConnectionTrait) -> Result<EmailAccount, String> {
    let account = get(db).await?;
    if account.saved {
        Ok(account)
    } else {
        Err("Configura la casella email in Impostazioni → Email prima di inviare".to_string())
    }
}

/// A preset provider always gets the preset server: only `Custom` stores what was typed.
pub async fn update(
    db: &impl ConnectionTrait,
    input: UpdateEmailAccountInput,
) -> Result<EmailAccount, String> {
    let account = normalise(input);
    validate_account(&account)?;
    let active = ActiveModel {
        id: Set(1),
        provider: Set(account.provider.as_str().to_string()),
        sender_address: Set(account.sender_address),
        sender_name: Set(account.sender_name),
        smtp_host: Set(account.smtp_host),
        smtp_port: Set(i32::from(account.smtp_port)),
        security: Set(account.security.as_str().to_string()),
        username: Set(account.username),
        bcc_self: Set(i32::from(account.bcc_self)),
        updated_at: Set(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
    };
    email_account_repository::save(db, active).await?;
    get(db).await
}

fn normalise(input: UpdateEmailAccountInput) -> EmailAccount {
    let sender_address = input.sender_address.trim().to_lowercase();
    let username = match input.username.trim() {
        "" => sender_address.clone(),
        typed => typed.to_string(),
    };
    let preset = input.provider.preset();
    let (smtp_host, smtp_port, security) = if input.provider == EmailProvider::Custom {
        (
            input.smtp_host.trim().to_lowercase(),
            input.smtp_port,
            input.security,
        )
    } else {
        (preset.host.to_string(), preset.port, preset.security)
    };
    EmailAccount {
        provider: input.provider,
        sender_address,
        sender_name: input.sender_name.trim().to_string(),
        smtp_host,
        smtp_port,
        security,
        username,
        bcc_self: input.bcc_self,
        saved: true,
    }
}

fn validate_account(account: &EmailAccount) -> Result<(), String> {
    validate::validate_required(&account.sender_address, "Indirizzo email")?;
    validate::validate_email(&account.sender_address)?;
    if account.sender_name.chars().count() > NAME_MAX_LENGTH {
        return Err(format!(
            "Nome del mittente: massimo {NAME_MAX_LENGTH} caratteri"
        ));
    }
    validate::validate_required(&account.smtp_host, "Server SMTP")?;
    let host_is_plain = account
        .smtp_host
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    if !host_is_plain || !account.smtp_host.contains('.') {
        return Err(format!("Server SMTP non valido: {}", account.smtp_host));
    }
    if account.smtp_port == 0 {
        return Err("Porta SMTP non valida".to_string());
    }
    validate::validate_required(&account.username, "Nome utente")
}

async fn proposed(db: &impl ConnectionTrait) -> Result<EmailAccount, String> {
    let profile = config_repository::find(db)
        .await
        .map_err(|e| e.to_string())?;
    let sender_address = profile
        .as_ref()
        .map(|p| p.pec_email.trim().to_lowercase())
        .unwrap_or_default();
    let sender_name = profile
        .map(|p| {
            format!("{} {} {}", p.title, p.first_name, p.last_name)
                .trim()
                .to_string()
        })
        .unwrap_or_default();
    let provider = if sender_address.is_empty() {
        EmailProvider::Psypec
    } else {
        EmailProvider::detect(&sender_address)
    };
    let preset = provider.preset();
    Ok(EmailAccount {
        provider,
        username: sender_address.clone(),
        sender_address,
        sender_name,
        smtp_host: preset.host.to_string(),
        smtp_port: preset.port,
        security: preset.security,
        bcc_self: false,
        saved: false,
    })
}

fn from_row(row: email_account::Model) -> Result<EmailAccount, String> {
    Ok(EmailAccount {
        provider: EmailProvider::parse(&row.provider)?,
        sender_address: row.sender_address,
        sender_name: row.sender_name,
        smtp_host: row.smtp_host,
        smtp_port: u16::try_from(row.smtp_port).map_err(|_| "Porta SMTP salvata non valida")?,
        security: EmailSecurity::parse(&row.security)?,
        username: row.username,
        bcc_self: row.bcc_self != 0,
        saved: true,
    })
}

#[cfg(test)]
#[path = "email_account_service_test.rs"]
mod tests;
