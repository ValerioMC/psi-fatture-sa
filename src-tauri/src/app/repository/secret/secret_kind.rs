/// The secrets the app keeps, each under its own entry of the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretKind {
    TsPassword,
    TsPincode,
    EmailPassword,
}

impl SecretKind {
    /// The entry name in the secrets file, also the authenticated data of its ciphertext.
    pub(super) fn account(&self) -> &'static str {
        match self {
            SecretKind::TsPassword => "password",
            SecretKind::TsPincode => "pincode",
            SecretKind::EmailPassword => "email_password",
        }
    }
}
