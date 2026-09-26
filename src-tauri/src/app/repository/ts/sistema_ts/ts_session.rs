use crate::app::model::ts::TsEnvironment;

/// Everything a call needs to authenticate, in clear; it never leaves memory.
#[derive(Clone)]
pub struct TsSession {
    pub environment: TsEnvironment,
    pub username: String,
    pub password: String,
    pub pincode: String,
}

/// Redacts the secrets, so a session can be logged or asserted on safely.
impl std::fmt::Debug for TsSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TsSession")
            .field("environment", &self.environment)
            .field("username", &self.username)
            .field("password", &"***")
            .field("pincode", &"***")
            .finish()
    }
}
