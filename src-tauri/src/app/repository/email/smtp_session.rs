use crate::app::model::email::EmailSecurity;

/// Everything needed to log in to the SMTP server, password included; built for one
/// call and never stored or serialised.
#[derive(Clone, PartialEq, Eq)]
pub struct SmtpSession {
    pub host: String,
    pub port: u16,
    pub security: EmailSecurity,
    pub username: String,
    pub password: String,
}

impl std::fmt::Debug for SmtpSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmtpSession")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("security", &self.security)
            .field("username", &self.username)
            .finish_non_exhaustive()
    }
}
