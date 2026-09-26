/// Encrypts one field of a Sistema TS request, returning it base64 encoded.
pub trait FieldCipher: Send + Sync {
    fn encrypt(&self, plain: &str) -> Result<String, String>;
}
