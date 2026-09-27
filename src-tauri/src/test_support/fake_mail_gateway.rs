use std::sync::Mutex;

use async_trait::async_trait;

use crate::app::repository::email::{MailGateway, MailGatewayError, OutgoingMail, SmtpSession};

/// Records what would have been sent; fails every call with `failure` when set.
#[derive(Default)]
pub struct FakeMailGateway {
    pub sent: Mutex<Vec<(SmtpSession, OutgoingMail)>>,
    pub checks: Mutex<Vec<SmtpSession>>,
    pub failure: Mutex<Option<MailGatewayError>>,
}

impl FakeMailGateway {
    pub fn failing(error: MailGatewayError) -> Self {
        FakeMailGateway {
            failure: Mutex::new(Some(error)),
            ..FakeMailGateway::default()
        }
    }

    pub fn sent(&self) -> Vec<(SmtpSession, OutgoingMail)> {
        self.sent.lock().unwrap().clone()
    }

    fn outcome(&self) -> Result<(), MailGatewayError> {
        match self.failure.lock().unwrap().clone() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

#[async_trait]
impl MailGateway for FakeMailGateway {
    async fn send(
        &self,
        session: &SmtpSession,
        mail: &OutgoingMail,
    ) -> Result<(), MailGatewayError> {
        self.outcome()?;
        self.sent
            .lock()
            .unwrap()
            .push((session.clone(), mail.clone()));
        Ok(())
    }

    async fn check(&self, session: &SmtpSession) -> Result<(), MailGatewayError> {
        self.checks.lock().unwrap().push(session.clone());
        self.outcome()
    }
}
