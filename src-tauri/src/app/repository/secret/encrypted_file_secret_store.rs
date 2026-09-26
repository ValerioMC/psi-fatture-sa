use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use hkdf::Hkdf;
use sha2::Sha256;

use super::secret_file::FILE_VERSION;
use super::{
    LoadedFile, MachineIdSource, SealedSecret, SecretFile, SecretKind, SecretStore,
    SecretStoreError,
};

const KEY_INFO: &[u8] = b"it.psifatture.sistema-ts/secrets/v1";
const NONCE_LENGTH: usize = 12;

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
            LoadedFile::Missing => SecretFile::empty(),
            LoadedFile::Corrupt => {
                log::warn!(target: "secret_store", path:? = self.path; "unreadable secrets file replaced");
                SecretFile::empty()
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
#[path = "encrypted_file_secret_store_test.rs"]
mod tests;
