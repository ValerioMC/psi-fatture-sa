use crate::app::common::AppError;
use crate::app::model::ts::{TsCredentialsStatus, TsSettings};
use crate::app::repository::secret::{SecretKind, SecretStore, SecretStoreError};
use crate::app::repository::ts::sistema_ts::TsSession;

const SECRET_MAX_LENGTH: usize = 64;

/// Reports which Sistema TS secrets are stored, without exposing them. An
/// unreadable one counts as missing, so the Impostazioni ask for it again.
pub fn status(store: &dyn SecretStore) -> Result<TsCredentialsStatus, AppError> {
    Ok(TsCredentialsStatus {
        password_configured: is_configured(store, SecretKind::TsPassword)?,
        pincode_configured: is_configured(store, SecretKind::TsPincode)?,
    })
}

/// Stores the PINCODE encrypted, replacing any previous one.
pub fn save_pincode(
    store: &dyn SecretStore,
    pincode: &str,
) -> Result<TsCredentialsStatus, AppError> {
    save(store, SecretKind::TsPincode, pincode.trim(), "PINCODE")
}

pub fn delete_pincode(store: &dyn SecretStore) -> Result<TsCredentialsStatus, AppError> {
    remove(store, SecretKind::TsPincode)
}

/// Stores the Sistema TS password encrypted. Kept as typed: spaces count.
pub fn save_password(
    store: &dyn SecretStore,
    password: &str,
) -> Result<TsCredentialsStatus, AppError> {
    save(store, SecretKind::TsPassword, password, "Password")
}

pub fn delete_password(store: &dyn SecretStore) -> Result<TsCredentialsStatus, AppError> {
    remove(store, SecretKind::TsPassword)
}

/// Everything a call needs, or the first missing piece named in plain words.
pub fn session(store: &dyn SecretStore, settings: &TsSettings) -> Result<TsSession, AppError> {
    if settings.username.trim().is_empty() || settings.vat_number.trim().is_empty() {
        return Err(AppError::Invalid(
            "Completa codice fiscale e partita IVA del Sistema TS nelle Impostazioni".to_string(),
        ));
    }
    let password = read(store, SecretKind::TsPassword)?.ok_or_else(|| {
        AppError::Invalid(
            "Manca la password del Sistema TS: inseriscila nelle Impostazioni".to_string(),
        )
    })?;
    let pincode = read(store, SecretKind::TsPincode)?.ok_or_else(|| {
        AppError::Invalid(
            "Manca il PINCODE del Sistema TS: inseriscilo nelle Impostazioni".to_string(),
        )
    })?;
    Ok(TsSession {
        environment: settings.environment,
        username: settings.username.clone(),
        password,
        pincode,
    })
}

fn read(store: &dyn SecretStore, kind: SecretKind) -> Result<Option<String>, AppError> {
    store.read(kind).map_err(AppError::from)
}

fn is_configured(store: &dyn SecretStore, kind: SecretKind) -> Result<bool, AppError> {
    match store.read(kind) {
        Ok(secret) => Ok(secret.is_some()),
        Err(SecretStoreError::Unreadable) => {
            log::warn!(target: "sistema_ts", secret:? = kind; "stored secret unreadable, reported as missing");
            Ok(false)
        }
        Err(e) => Err(e.into()),
    }
}

fn save(
    store: &dyn SecretStore,
    kind: SecretKind,
    secret: &str,
    label: &str,
) -> Result<TsCredentialsStatus, AppError> {
    validate_secret(secret, label, kind == SecretKind::TsPincode)?;
    store.write(kind, secret)?;
    status(store)
}

fn remove(store: &dyn SecretStore, kind: SecretKind) -> Result<TsCredentialsStatus, AppError> {
    store.remove(kind)?;
    status(store)
}

fn validate_secret(secret: &str, label: &str, forbid_spaces: bool) -> Result<(), AppError> {
    if secret.trim().is_empty() {
        return Err(AppError::Invalid(format!("{label}: campo obbligatorio")));
    }
    if forbid_spaces && secret.chars().any(char::is_whitespace) {
        return Err(AppError::Invalid(format!(
            "{label}: non può contenere spazi"
        )));
    }
    if secret.chars().count() > SECRET_MAX_LENGTH {
        return Err(AppError::Invalid(format!(
            "{label}: massimo {SECRET_MAX_LENGTH} caratteri"
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "ts_credential_service_test.rs"]
mod tests;
