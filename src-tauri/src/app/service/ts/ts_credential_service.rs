use crate::app::model::ts::{TsCredentialsStatus, TsSettings};
use crate::app::repository::secret::{SecretKind, SecretStore, SecretStoreError};
use crate::app::repository::ts::sistema_ts::TsSession;

const SECRET_MAX_LENGTH: usize = 64;

/// Reports which Sistema TS secrets are stored, without exposing them. An
/// unreadable one counts as missing, so the Impostazioni ask for it again.
pub fn status(store: &dyn SecretStore) -> Result<TsCredentialsStatus, String> {
    Ok(TsCredentialsStatus {
        password_configured: is_configured(store, SecretKind::TsPassword)?,
        pincode_configured: is_configured(store, SecretKind::TsPincode)?,
    })
}

/// Stores the PINCODE encrypted, replacing any previous one.
pub fn save_pincode(store: &dyn SecretStore, pincode: &str) -> Result<TsCredentialsStatus, String> {
    save(store, SecretKind::TsPincode, pincode.trim(), "PINCODE")
}

pub fn delete_pincode(store: &dyn SecretStore) -> Result<TsCredentialsStatus, String> {
    remove(store, SecretKind::TsPincode)
}

/// Stores the Sistema TS password encrypted. Kept as typed: spaces count.
pub fn save_password(
    store: &dyn SecretStore,
    password: &str,
) -> Result<TsCredentialsStatus, String> {
    save(store, SecretKind::TsPassword, password, "Password")
}

pub fn delete_password(store: &dyn SecretStore) -> Result<TsCredentialsStatus, String> {
    remove(store, SecretKind::TsPassword)
}

/// Everything a call needs, or the first missing piece named in plain words.
pub fn session(store: &dyn SecretStore, settings: &TsSettings) -> Result<TsSession, String> {
    if settings.username.trim().is_empty() || settings.vat_number.trim().is_empty() {
        return Err(
            "Completa codice fiscale e partita IVA del Sistema TS nelle Impostazioni".to_string(),
        );
    }
    let password = read(store, SecretKind::TsPassword)?
        .ok_or("Manca la password del Sistema TS: inseriscila nelle Impostazioni")?;
    let pincode = read(store, SecretKind::TsPincode)?
        .ok_or("Manca il PINCODE del Sistema TS: inseriscilo nelle Impostazioni")?;
    Ok(TsSession {
        environment: settings.environment,
        username: settings.username.clone(),
        password,
        pincode,
    })
}

fn read(store: &dyn SecretStore, kind: SecretKind) -> Result<Option<String>, String> {
    store.read(kind).map_err(|e| e.to_string())
}

fn is_configured(store: &dyn SecretStore, kind: SecretKind) -> Result<bool, String> {
    match store.read(kind) {
        Ok(secret) => Ok(secret.is_some()),
        Err(SecretStoreError::Unreadable) => {
            log::warn!(target: "sistema_ts", secret:? = kind; "stored secret unreadable, reported as missing");
            Ok(false)
        }
        Err(e) => Err(e.to_string()),
    }
}

fn save(
    store: &dyn SecretStore,
    kind: SecretKind,
    secret: &str,
    label: &str,
) -> Result<TsCredentialsStatus, String> {
    validate_secret(secret, label, kind == SecretKind::TsPincode)?;
    store.write(kind, secret).map_err(|e| e.to_string())?;
    status(store)
}

fn remove(store: &dyn SecretStore, kind: SecretKind) -> Result<TsCredentialsStatus, String> {
    store.remove(kind).map_err(|e| e.to_string())?;
    status(store)
}

fn validate_secret(secret: &str, label: &str, forbid_spaces: bool) -> Result<(), String> {
    if secret.trim().is_empty() {
        return Err(format!("{label}: campo obbligatorio"));
    }
    if forbid_spaces && secret.chars().any(char::is_whitespace) {
        return Err(format!("{label}: non può contenere spazi"));
    }
    if secret.chars().count() > SECRET_MAX_LENGTH {
        return Err(format!("{label}: massimo {SECRET_MAX_LENGTH} caratteri"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "ts_credential_service_test.rs"]
mod tests;
