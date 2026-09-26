use aes_gcm::aead::rand_core::RngCore;

use super::*;

struct FixedMachineId(&'static str);

impl MachineIdSource for FixedMachineId {
    fn machine_id(&self) -> Result<String, SecretStoreError> {
        Ok(self.0.to_owned())
    }
}

struct TempPath(PathBuf);

impl TempPath {
    fn new() -> Self {
        let name = format!("psifatture-secrets-{}.json", OsRng.next_u64());
        Self(std::env::temp_dir().join(name))
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn store_on(path: &TempPath, machine: &'static str) -> EncryptedFileSecretStore {
    EncryptedFileSecretStore::new(path.0.clone(), Box::new(FixedMachineId(machine)))
}

#[test]
fn missing_file_reads_as_empty() {
    let path = TempPath::new();
    assert!(store_on(&path, "mac-a")
        .read(SecretKind::TsPincode)
        .unwrap()
        .is_none());
}

#[test]
fn round_trips_each_secret_separately() {
    let path = TempPath::new();
    let store = store_on(&path, "mac-a");
    store.write(SecretKind::TsPassword, " Salve 123").unwrap();
    store.write(SecretKind::TsPincode, "3489543096").unwrap();
    store.write(SecretKind::TsPincode, "1111").unwrap();

    let reopened = store_on(&path, "mac-a");
    assert_eq!(
        reopened.read(SecretKind::TsPassword).unwrap().as_deref(),
        Some(" Salve 123")
    );
    assert_eq!(
        reopened.read(SecretKind::TsPincode).unwrap().as_deref(),
        Some("1111")
    );
}

#[test]
fn never_writes_secrets_in_clear() {
    let path = TempPath::new();
    store_on(&path, "mac-a")
        .write(SecretKind::TsPassword, "Salve123")
        .unwrap();
    let raw = fs::read_to_string(&path.0).unwrap();
    assert!(!raw.contains("Salve123"));
    assert!(!raw.contains(&BASE64.encode("Salve123")));
}

#[cfg(unix)]
#[test]
fn file_is_readable_by_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let path = TempPath::new();
    store_on(&path, "mac-a")
        .write(SecretKind::TsPincode, "1234")
        .unwrap();
    let mode = fs::metadata(&path.0).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn another_computer_cannot_read_the_file() {
    let path = TempPath::new();
    store_on(&path, "mac-a")
        .write(SecretKind::TsPincode, "1234")
        .unwrap();
    assert!(matches!(
        store_on(&path, "mac-b").read(SecretKind::TsPincode),
        Err(SecretStoreError::Unreadable)
    ));
}

#[test]
fn swapped_entries_are_rejected() {
    let path = TempPath::new();
    let store = store_on(&path, "mac-a");
    store.write(SecretKind::TsPassword, "Salve123").unwrap();
    let mut file: SecretFile = serde_json::from_slice(&fs::read(&path.0).unwrap()).unwrap();
    let password = file.entries.remove("password").unwrap();
    file.entries.insert("pincode".to_owned(), password);
    fs::write(&path.0, serde_json::to_vec(&file).unwrap()).unwrap();

    assert!(matches!(
        store.read(SecretKind::TsPincode),
        Err(SecretStoreError::Unreadable)
    ));
}

#[test]
fn corrupt_file_is_unreadable_until_rewritten() {
    let path = TempPath::new();
    fs::write(&path.0, b"not json").unwrap();
    let store = store_on(&path, "mac-a");
    assert!(matches!(
        store.read(SecretKind::TsPincode),
        Err(SecretStoreError::Unreadable)
    ));
    store.remove(SecretKind::TsPincode).unwrap();

    store.write(SecretKind::TsPincode, "1234").unwrap();
    assert_eq!(
        store.read(SecretKind::TsPincode).unwrap().as_deref(),
        Some("1234")
    );
}

#[test]
fn remove_keeps_the_other_secret_and_is_idempotent() {
    let path = TempPath::new();
    let store = store_on(&path, "mac-a");
    store.write(SecretKind::TsPassword, "Salve123").unwrap();
    store.write(SecretKind::TsPincode, "1234").unwrap();
    store.remove(SecretKind::TsPincode).unwrap();
    store.remove(SecretKind::TsPincode).unwrap();

    assert!(store.read(SecretKind::TsPincode).unwrap().is_none());
    assert_eq!(
        store.read(SecretKind::TsPassword).unwrap().as_deref(),
        Some("Salve123")
    );
}
