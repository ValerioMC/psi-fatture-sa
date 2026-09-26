//! Field encryption required by the Sistema TS: the PINCODE and every codice
//! fiscale travel as RSA PKCS#1 v1.5 ciphertext, base64 encoded, under the
//! public key of the Sogei `SanitelCF` certificate (expires 23/01/2027).

use base64::{engine::general_purpose::STANDARD, Engine};
use rsa::{pkcs8::DecodePublicKey, Pkcs1v15Encrypt, RsaPublicKey};
use x509_cert::{
    der::{DecodePem, Encode},
    Certificate,
};

use super::FieldCipher;

const SANITEL_CERTIFICATE: &str = include_str!("../../../../../resources/sistema_ts/SanitelCF.pem");
pub struct SanitelCipher {
    key: RsaPublicKey,
}

impl SanitelCipher {
    pub fn from_bundled_certificate() -> Result<Self, String> {
        let certificate = Certificate::from_pem(SANITEL_CERTIFICATE.as_bytes())
            .map_err(|e| format!("Certificato SanitelCF non leggibile: {e}"))?;
        let spki = certificate
            .tbs_certificate
            .subject_public_key_info
            .to_der()
            .map_err(|e| format!("Chiave SanitelCF non leggibile: {e}"))?;
        let key = RsaPublicKey::from_public_key_der(&spki)
            .map_err(|e| format!("Chiave SanitelCF non RSA: {e}"))?;
        Ok(Self { key })
    }
}

impl FieldCipher for SanitelCipher {
    fn encrypt(&self, plain: &str) -> Result<String, String> {
        let ciphertext = self
            .key
            .encrypt(&mut rand::thread_rng(), Pkcs1v15Encrypt, plain.as_bytes())
            .map_err(|e| format!("Cifratura non riuscita: {e}"))?;
        Ok(STANDARD.encode(ciphertext))
    }
}

#[cfg(test)]
#[path = "sanitel_cipher_test.rs"]
mod tests;
