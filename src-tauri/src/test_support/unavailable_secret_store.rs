use crate::app::repository::secret::{SecretKind, SecretStore, SecretStoreError};

/// Fails every call, as a store whose file cannot be reached.
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
