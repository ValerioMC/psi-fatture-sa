use super::SecretStoreError;

/// Where the encryption key comes from: it is bound to this computer, so a
/// copy of the data folder is useless elsewhere.
pub trait MachineIdSource: Send + Sync {
    fn machine_id(&self) -> Result<String, SecretStoreError>;
}
