use async_trait::async_trait;

use super::{MailGatewayError, OutgoingMail, SmtpSession};

/// The SMTP server, behind a seam so services can be tested without a network.
#[async_trait]
pub trait MailGateway: Send + Sync {
    async fn send(
        &self,
        session: &SmtpSession,
        mail: &OutgoingMail,
    ) -> Result<(), MailGatewayError>;

    /// Connects and logs in, sending nothing.
    async fn check(&self, session: &SmtpSession) -> Result<(), MailGatewayError>;
}
