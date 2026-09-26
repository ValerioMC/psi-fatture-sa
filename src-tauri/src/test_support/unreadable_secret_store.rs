use crate::app::repository::secret::{SecretKind, SecretStore, SecretStoreError};

/// Reads fail as they would after the file moved to another computer.
pub struct UnreadableSecretStore;

impl SecretStore for UnreadableSecretStore {
    fn read(&self, _kind: SecretKind) -> Result<Option<String>, SecretStoreError> {
        Err(SecretStoreError::Unreadable)
    }

    fn write(&self, _kind: SecretKind, _secret: &str) -> Result<(), SecretStoreError> {
        Ok(())
    }

    fn remove(&self, _kind: SecretKind) -> Result<(), SecretStoreError> {
        Ok(())
    }
}
