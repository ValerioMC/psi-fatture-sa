use std::collections::HashMap;
use std::sync::Mutex;

use crate::app::repository::secret::{SecretKind, SecretStore, SecretStoreError};

/// Keeps secrets in a map, for services that only need a working store.
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
