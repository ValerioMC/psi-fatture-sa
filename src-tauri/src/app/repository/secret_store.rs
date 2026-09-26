//! Credentials kept in the OS keychain (macOS Keychain, Windows Credential
//! Manager), never on disk. `SecretStore` is the seam that lets tests run
//! against memory instead of prompting the real keychain.

use thiserror::Error;

const KEYRING_SERVICE: &str = "it.psifatture.sistema-ts";

/// The secrets the app keeps, each under its own keychain account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretKind {
    TsPassword,
    TsPincode,
}

impl SecretKind {
    fn account(&self) -> &'static str {
        match self {
            SecretKind::TsPassword => "password",
            SecretKind::TsPincode => "pincode",
        }
    }
}

#[derive(Debug, Error)]
pub enum SecretStoreError {
    #[error("Portachiavi di sistema non disponibile: {0}")]
    Unavailable(String),
}

pub trait SecretStore: Send + Sync {
    fn read(&self, kind: SecretKind) -> Result<Option<String>, SecretStoreError>;
    fn write(&self, kind: SecretKind, secret: &str) -> Result<(), SecretStoreError>;
    /// Removes the secret; removing one that is not there is not an error.
    fn remove(&self, kind: SecretKind) -> Result<(), SecretStoreError>;
}

pub struct KeyringSecretStore;

impl KeyringSecretStore {
    fn entry(kind: SecretKind) -> Result<keyring::Entry, SecretStoreError> {
        keyring::Entry::new(KEYRING_SERVICE, kind.account())
            .map_err(|e| SecretStoreError::Unavailable(e.to_string()))
    }
}

impl SecretStore for KeyringSecretStore {
    fn read(&self, kind: SecretKind) -> Result<Option<String>, SecretStoreError> {
        match Self::entry(kind)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecretStoreError::Unavailable(e.to_string())),
        }
    }

    fn write(&self, kind: SecretKind, secret: &str) -> Result<(), SecretStoreError> {
        Self::entry(kind)?
            .set_password(secret)
            .map_err(|e| SecretStoreError::Unavailable(e.to_string()))
    }

    fn remove(&self, kind: SecretKind) -> Result<(), SecretStoreError> {
        match Self::entry(kind)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(SecretStoreError::Unavailable(e.to_string())),
        }
    }
}

#[cfg(test)]
pub mod testing {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    pub struct InMemorySecretStore {
        secrets: Mutex<HashMap<SecretKind, String>>,
    }

    impl SecretStore for InMemorySecretStore {
        fn read(&self, kind: SecretKind) -> Result<Option<String>, SecretStoreError> {
            Ok(self.secrets.lock().unwrap().get(&kind).cloned())
        }

        fn write(&self, kind: SecretKind, secret: &str) -> Result<(), SecretStoreError> {
            self.secrets.lock().unwrap().insert(kind, secret.to_owned());
            Ok(())
        }

        fn remove(&self, kind: SecretKind) -> Result<(), SecretStoreError> {
            self.secrets.lock().unwrap().remove(&kind);
            Ok(())
        }
    }

    pub struct UnavailableSecretStore;

    impl SecretStore for UnavailableSecretStore {
        fn read(&self, _kind: SecretKind) -> Result<Option<String>, SecretStoreError> {
            Err(SecretStoreError::Unavailable("locked".to_string()))
        }

        fn write(&self, _kind: SecretKind, _secret: &str) -> Result<(), SecretStoreError> {
            Err(SecretStoreError::Unavailable("locked".to_string()))
        }

        fn remove(&self, _kind: SecretKind) -> Result<(), SecretStoreError> {
            Err(SecretStoreError::Unavailable("locked".to_string()))
        }
    }
}
