use std::collections::BTreeMap;

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::OsRng;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::{Deserialize, Serialize};

use super::SealedSecret;

pub(super) const FILE_VERSION: u32 = 1;
const SALT_LENGTH: usize = 32;

/// The JSON document on disk: a random salt and one sealed entry per secret.
#[derive(Serialize, Deserialize)]
pub(super) struct SecretFile {
    pub(super) version: u32,
    pub(super) salt: String,
    pub(super) entries: BTreeMap<String, SealedSecret>,
}

impl SecretFile {
    pub(super) fn empty() -> Self {
        let mut salt = [0u8; SALT_LENGTH];
        OsRng.fill_bytes(&mut salt);
        Self {
            version: FILE_VERSION,
            salt: BASE64.encode(salt),
            entries: BTreeMap::new(),
        }
    }
}
