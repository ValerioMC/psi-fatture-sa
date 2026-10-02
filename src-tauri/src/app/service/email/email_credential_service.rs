use crate::app::common::AppError;
use crate::app::model::email::{EmailAccount, EmailCredentialsStatus};
use crate::app::repository::email::SmtpSession;
use crate::app::repository::secret::{SecretKind, SecretStore, SecretStoreError};

const PASSWORD_MAX_LENGTH: usize = 128;

/// Whether the mailbox password is stored. An unreadable one counts as missing,
/// so the Impostazioni ask for it again.
pub fn status(store: &dyn SecretStore) -> Result<EmailCredentialsStatus, AppError> {
    let password_configured = match store.read(SecretKind::EmailPassword) {
        Ok(secret) => secret.is_some(),
        Err(SecretStoreError::Unreadable) => {
            log::warn!(target: "email", "stored mailbox password unreadable, reported as missing");
            false
        }
        Err(e) => return Err(e.into()),
    };
    Ok(EmailCredentialsStatus {
        password_configured,
    })
}

/// Stores the password encrypted, as typed: spaces count.
pub fn save_password(
    store: &dyn SecretStore,
    password: &str,
) -> Result<EmailCredentialsStatus, AppError> {
    if password.trim().is_empty() {
        return Err(AppError::Invalid(
            "Password della casella: campo obbligatorio".to_string(),
        ));
    }
    if password.chars().count() > PASSWORD_MAX_LENGTH {
        return Err(AppError::Invalid(format!(
            "Password della casella: massimo {PASSWORD_MAX_LENGTH} caratteri"
        )));
    }
    store.write(SecretKind::EmailPassword, password)?;
    status(store)
}

pub fn delete_password(store: &dyn SecretStore) -> Result<EmailCredentialsStatus, AppError> {
    store.remove(SecretKind::EmailPassword)?;
    status(store)
}

/// The login for one SMTP call, or what is missing in plain words.
pub fn session(store: &dyn SecretStore, account: &EmailAccount) -> Result<SmtpSession, AppError> {
    let password = store.read(SecretKind::EmailPassword)?.ok_or_else(|| {
        AppError::Invalid(
            "Manca la password della casella email: inseriscila in Impostazioni → Email"
                .to_string(),
        )
    })?;
    Ok(SmtpSession {
        host: account.smtp_host.clone(),
        port: account.smtp_port,
        security: account.security,
        username: account.username.clone(),
        password,
    })
}

#[cfg(test)]
#[path = "email_credential_service_test.rs"]
mod tests;
