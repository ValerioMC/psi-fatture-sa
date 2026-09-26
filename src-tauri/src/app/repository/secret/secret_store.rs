use super::{SecretKind, SecretStoreError};

pub trait SecretStore: Send + Sync {
    fn read(&self, kind: SecretKind) -> Result<Option<String>, SecretStoreError>;
    fn write(&self, kind: SecretKind, secret: &str) -> Result<(), SecretStoreError>;
    /// Removes the secret; removing one that is not there is not an error.
    fn remove(&self, kind: SecretKind) -> Result<(), SecretStoreError>;
}
