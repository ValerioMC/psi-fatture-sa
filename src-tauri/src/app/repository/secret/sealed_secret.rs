use serde::{Deserialize, Serialize};

/// One secret as AES-GCM ciphertext with its nonce, both base64 encoded.
#[derive(Serialize, Deserialize)]
pub(super) struct SealedSecret {
    pub(super) nonce: String,
    pub(super) ciphertext: String,
}
