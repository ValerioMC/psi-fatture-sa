use crate::app::model::ts::{TsCredentialsStatus, TsSettings};
use crate::app::repository::secret_store::{SecretKind, SecretStore};
use crate::app::repository::sistema_ts::gateway::TsSession;

const SECRET_MAX_LENGTH: usize = 64;

/// Reports which Sistema TS secrets are stored, without exposing them.
pub fn status(store: &dyn SecretStore) -> Result<TsCredentialsStatus, String> {
    Ok(TsCredentialsStatus {
        password_configured: read(store, SecretKind::TsPassword)?.is_some(),
        pincode_configured: read(store, SecretKind::TsPincode)?.is_some(),
    })
}

/// Stores the PINCODE in the OS keychain, replacing any previous one.
pub fn save_pincode(store: &dyn SecretStore, pincode: &str) -> Result<TsCredentialsStatus, String> {
    save(store, SecretKind::TsPincode, pincode.trim(), "PINCODE")
}

pub fn delete_pincode(store: &dyn SecretStore) -> Result<TsCredentialsStatus, String> {
    remove(store, SecretKind::TsPincode)
}

/// Stores the Sistema TS password in the OS keychain. Kept as typed: spaces count.
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
mod tests {
    use super::*;
    use crate::app::model::ts::TsEnvironment;
    use crate::app::repository::secret_store::testing::{
        InMemorySecretStore, UnavailableSecretStore,
    };

    fn settings() -> TsSettings {
        TsSettings {
            environment: TsEnvironment::Test,
            username: "MTOMRA66A41G224M".to_string(),
            vat_number: "65498732105".to_string(),
        }
    }

    #[test]
    fn reports_missing_secrets_on_empty_store() {
        let store = InMemorySecretStore::default();
        let status = status(&store).unwrap();
        assert!(!status.pincode_configured);
        assert!(!status.password_configured);
    }

    #[test]
    fn saves_trimmed_pincode_and_untrimmed_password() {
        let store = InMemorySecretStore::default();
        save_pincode(&store, "  1234567890 ").unwrap();
        let result = save_password(&store, " Salve 123").unwrap();
        assert!(result.pincode_configured && result.password_configured);
        assert_eq!(
            store.read(SecretKind::TsPincode).unwrap().as_deref(),
            Some("1234567890")
        );
        assert_eq!(
            store.read(SecretKind::TsPassword).unwrap().as_deref(),
            Some(" Salve 123")
        );
    }

    #[test]
    fn replaces_existing_pincode() {
        let store = InMemorySecretStore::default();
        save_pincode(&store, "1111").unwrap();
        save_pincode(&store, "2222").unwrap();
        assert_eq!(
            store.read(SecretKind::TsPincode).unwrap().as_deref(),
            Some("2222")
        );
    }

    #[test]
    fn rejects_blank_spaced_or_oversized_secrets() {
        let store = InMemorySecretStore::default();
        assert!(save_pincode(&store, "   ").is_err());
        assert!(save_pincode(&store, "12 34").is_err());
        assert!(save_pincode(&store, &"9".repeat(SECRET_MAX_LENGTH + 1)).is_err());
        assert!(save_password(&store, "  ").is_err());
        assert!(!status(&store).unwrap().pincode_configured);
    }

    #[test]
    fn delete_is_idempotent() {
        let store = InMemorySecretStore::default();
        save_pincode(&store, "1234").unwrap();
        assert!(!delete_pincode(&store).unwrap().pincode_configured);
        assert!(!delete_pincode(&store).unwrap().pincode_configured);
        assert!(!delete_password(&store).unwrap().password_configured);
    }

    #[test]
    fn surfaces_keychain_failures() {
        let store = UnavailableSecretStore;
        assert!(status(&store).unwrap_err().contains("Portachiavi"));
        assert!(save_pincode(&store, "1234").is_err());
        assert!(delete_password(&store).is_err());
    }

    #[test]
    fn status_never_serializes_the_secrets() {
        let store = InMemorySecretStore::default();
        save_pincode(&store, "1234567890").unwrap();
        save_password(&store, "Salve123").unwrap();
        let json = serde_json::to_string(&status(&store).unwrap()).unwrap();
        assert!(!json.contains("1234567890") && !json.contains("Salve123"));
    }

    #[test]
    fn session_names_the_first_missing_piece() {
        let store = InMemorySecretStore::default();
        assert!(session(&store, &settings())
            .unwrap_err()
            .contains("password"));
        save_password(&store, "Salve123").unwrap();
        assert!(session(&store, &settings())
            .unwrap_err()
            .contains("PINCODE"));
        save_pincode(&store, "3489543096").unwrap();

        let mut blank = settings();
        blank.vat_number.clear();
        assert!(session(&store, &blank).unwrap_err().contains("partita IVA"));

        let ready = session(&store, &settings()).unwrap();
        assert_eq!(ready.username, "MTOMRA66A41G224M");
        assert_eq!(ready.environment, TsEnvironment::Test);
    }
}
