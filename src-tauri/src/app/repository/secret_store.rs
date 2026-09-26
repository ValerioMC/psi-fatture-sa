//! Credentials kept by the app itself in an encrypted file next to the
//! database: never in clear on disk, never in the OS keychain (which would
//! prompt for permission). `SecretStore` is the seam that lets tests run
//! against memory.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;

const FILE_VERSION: u32 = 1;
const KEY_INFO: &[u8] = b"it.psifatture.sistema-ts/secrets/v1";
const SALT_LENGTH: usize = 32;
const NONCE_LENGTH: usize = 12;

/// The secrets the app keeps, each under its own entry of the file.
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
    #[error("Archivio credenziali non disponibile: {0}")]
    Unavailable(String),
    #[error("Credenziale salvata non leggibile su questo computer: inseriscila di nuovo nelle Impostazioni")]
    Unreadable,
}

pub trait SecretStore: Send + Sync {
    fn read(&self, kind: SecretKind) -> Result<Option<String>, SecretStoreError>;
    fn write(&self, kind: SecretKind, secret: &str) -> Result<(), SecretStoreError>;
    /// Removes the secret; removing one that is not there is not an error.
    fn remove(&self, kind: SecretKind) -> Result<(), SecretStoreError>;
}

/// Where the encryption key comes from: it is bound to this computer, so a
/// copy of the data folder is useless elsewhere.
pub trait MachineIdSource: Send + Sync {
    fn machine_id(&self) -> Result<String, SecretStoreError>;
}

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

#[derive(Serialize, Deserialize)]
struct SecretFile {
    version: u32,
    salt: String,
    entries: BTreeMap<String, SealedSecret>,
}

#[derive(Serialize, Deserialize)]
struct SealedSecret {
    nonce: String,
    ciphertext: String,
}

enum LoadedFile {
    Missing,
    Corrupt,
    Parsed(SecretFile),
}

/// AES-256-GCM entries in one JSON file. The key is derived with HKDF-SHA256
/// from the machine identifier and a random per-file salt; each entry's name
/// is authenticated data, so entries cannot be swapped.
pub struct EncryptedFileSecretStore {
    path: PathBuf,
    machine: Box<dyn MachineIdSource>,
    file_lock: Mutex<()>,
}

impl EncryptedFileSecretStore {
    pub fn new(path: PathBuf, machine: Box<dyn MachineIdSource>) -> Self {
        Self {
            path,
            machine,
            file_lock: Mutex::new(()),
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, ()>, SecretStoreError> {
        self.file_lock
            .lock()
            .map_err(|_| SecretStoreError::Unavailable("archivio bloccato".to_string()))
    }

    fn load(&self) -> Result<LoadedFile, SecretStoreError> {
        let raw = match fs::read(&self.path) {
            Ok(raw) => raw,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(LoadedFile::Missing),
            Err(e) => return Err(SecretStoreError::Unavailable(e.to_string())),
        };
        match serde_json::from_slice::<SecretFile>(&raw) {
            Ok(file) if file.version == FILE_VERSION => Ok(LoadedFile::Parsed(file)),
            _ => Ok(LoadedFile::Corrupt),
        }
    }

    /// Writes through a temporary file and a rename, so a crash never leaves half a file.
    fn store(&self, file: &SecretFile) -> Result<(), SecretStoreError> {
        let unavailable = |e: std::io::Error| SecretStoreError::Unavailable(e.to_string());
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(unavailable)?;
        }
        let json = serde_json::to_vec_pretty(file)
            .map_err(|e| SecretStoreError::Unavailable(e.to_string()))?;
        let temporary = self.path.with_extension("json.tmp");
        let mut handle = owner_only_file(&temporary).map_err(unavailable)?;
        handle.write_all(&json).map_err(unavailable)?;
        handle.sync_all().map_err(unavailable)?;
        fs::rename(&temporary, &self.path).map_err(unavailable)
    }

    fn cipher(&self, salt: &str) -> Result<Aes256Gcm, SecretStoreError> {
        let salt = BASE64
            .decode(salt)
            .map_err(|_| SecretStoreError::Unreadable)?;
        let machine_id = self.machine.machine_id()?;
        let mut key = [0u8; 32];
        Hkdf::<Sha256>::new(Some(&salt), machine_id.as_bytes())
            .expand(KEY_INFO, &mut key)
            .map_err(|e| SecretStoreError::Unavailable(e.to_string()))?;
        Ok(Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key)))
    }

    fn open(
        &self,
        file: &SecretFile,
        kind: SecretKind,
    ) -> Result<Option<String>, SecretStoreError> {
        let Some(sealed) = file.entries.get(kind.account()) else {
            return Ok(None);
        };
        let cipher = self.cipher(&file.salt)?;
        let nonce = BASE64
            .decode(&sealed.nonce)
            .map_err(|_| SecretStoreError::Unreadable)?;
        let ciphertext = BASE64
            .decode(&sealed.ciphertext)
            .map_err(|_| SecretStoreError::Unreadable)?;
        if nonce.len() != NONCE_LENGTH {
            return Err(SecretStoreError::Unreadable);
        }
        let payload = Payload {
            msg: &ciphertext,
            aad: kind.account().as_bytes(),
        };
        let plain = cipher
            .decrypt(Nonce::from_slice(&nonce), payload)
            .map_err(|_| SecretStoreError::Unreadable)?;
        String::from_utf8(plain)
            .map(Some)
            .map_err(|_| SecretStoreError::Unreadable)
    }

    fn seal(
        &self,
        salt: &str,
        kind: SecretKind,
        secret: &str,
    ) -> Result<SealedSecret, SecretStoreError> {
        let cipher = self.cipher(salt)?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let payload = Payload {
            msg: secret.as_bytes(),
            aad: kind.account().as_bytes(),
        };
        let ciphertext = cipher
            .encrypt(&nonce, payload)
            .map_err(|e| SecretStoreError::Unavailable(e.to_string()))?;
        Ok(SealedSecret {
            nonce: BASE64.encode(nonce),
            ciphertext: BASE64.encode(ciphertext),
        })
    }
}

impl SecretStore for EncryptedFileSecretStore {
    fn read(&self, kind: SecretKind) -> Result<Option<String>, SecretStoreError> {
        let _guard = self.lock()?;
        match self.load()? {
            LoadedFile::Missing => Ok(None),
            LoadedFile::Corrupt => Err(SecretStoreError::Unreadable),
            LoadedFile::Parsed(file) => self.open(&file, kind),
        }
    }

    /// A corrupt file holds nothing readable, so it is replaced by a fresh one.
    fn write(&self, kind: SecretKind, secret: &str) -> Result<(), SecretStoreError> {
        let _guard = self.lock()?;
        let mut file = match self.load()? {
            LoadedFile::Parsed(file) => file,
            LoadedFile::Missing => empty_file(),
            LoadedFile::Corrupt => {
                log::warn!(target: "secret_store", path:? = self.path; "unreadable secrets file replaced");
                empty_file()
            }
        };
        let sealed = self.seal(&file.salt, kind, secret)?;
        file.entries.insert(kind.account().to_owned(), sealed);
        self.store(&file)
    }

    fn remove(&self, kind: SecretKind) -> Result<(), SecretStoreError> {
        let _guard = self.lock()?;
        match self.load()? {
            LoadedFile::Missing | LoadedFile::Corrupt => Ok(()),
            LoadedFile::Parsed(mut file) => {
                if file.entries.remove(kind.account()).is_none() {
                    return Ok(());
                }
                self.store(&file)
            }
        }
    }
}

fn empty_file() -> SecretFile {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    SecretFile {
        version: FILE_VERSION,
        salt: BASE64.encode(salt),
        entries: BTreeMap::new(),
    }
}

#[cfg(unix)]
fn owner_only_file(path: &Path) -> std::io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn owner_only_file(path: &Path) -> std::io::Result<fs::File> {
    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
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
}

#[cfg(test)]
mod tests {
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
}
