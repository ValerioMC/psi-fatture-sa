use std::sync::OnceLock;

use super::{MachineIdSource, SecretStoreError};

/// The OS machine identifier (IOPlatformUUID on macOS, MachineGuid on
/// Windows). Only a successful read is cached, so a transient failure retries.
#[derive(Default)]
pub struct OsMachineId {
    cached: OnceLock<String>,
}

impl MachineIdSource for OsMachineId {
    fn machine_id(&self) -> Result<String, SecretStoreError> {
        if let Some(id) = self.cached.get() {
            return Ok(id.clone());
        }
        let id = machine_uid::get()
            .map_err(|e| SecretStoreError::Unavailable(e.to_string()))?
            .trim()
            .to_owned();
        if id.is_empty() {
            return Err(SecretStoreError::Unavailable(
                "identificativo del computer vuoto".to_string(),
            ));
        }
        Ok(self.cached.get_or_init(|| id).clone())
    }
}
