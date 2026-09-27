use std::time::Duration;

use async_trait::async_trait;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::AsyncSmtpTransportBuilder;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use super::{MailGateway, MailGatewayError, OutgoingMail, SmtpSession};
use crate::app::model::email::EmailSecurity;

/// Long enough for a slow PEC server to accept a PDF, short enough not to leave the button hanging.
const TIMEOUT: Duration = Duration::from_secs(30);

/// Sends over SMTP with lettre and rustls; one connection per call, closed after it.
#[derive(Default)]
pub struct LettreMailGateway;

#[async_trait]
impl MailGateway for LettreMailGateway {
    async fn send(
        &self,
        session: &SmtpSession,
        mail: &OutgoingMail,
    ) -> Result<(), MailGatewayError> {
        let message = build_message(mail)?;
        let transport = transport(session)?;
        transport.send(message).await.map_err(classify)?;
        log::info!(target: "email", host = session.host.as_str(); "invoice email accepted by the server");
        Ok(())
    }

    async fn check(&self, session: &SmtpSession) -> Result<(), MailGatewayError> {
        let connected = transport(session)?
            .test_connection()
            .await
            .map_err(classify)?;
        if connected {
            Ok(())
        } else {
            Err(MailGatewayError::Connection(
                "connessione chiusa dal server".to_string(),
            ))
        }
    }
}

fn transport(
    session: &SmtpSession,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, MailGatewayError> {
    let host = session.host.trim();
    let builder: AsyncSmtpTransportBuilder = match session.security {
        EmailSecurity::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(host),
        EmailSecurity::Starttls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host),
    }
    .map_err(|e| MailGatewayError::Connection(e.to_string()))?;
    Ok(builder
        .port(session.port)
        .credentials(Credentials::new(
            session.username.clone(),
            session.password.clone(),
        ))
        .timeout(Some(TIMEOUT))
        .build())
}

fn mailbox(name: &str, address: &str) -> Result<Mailbox, MailGatewayError> {
    let address = address.trim().parse().map_err(|_| {
        MailGatewayError::InvalidMessage(format!("indirizzo non valido: {}", address.trim()))
    })?;
    let name = Some(name.trim().to_string()).filter(|name| !name.is_empty());
    Ok(Mailbox::new(name, address))
}

fn build_message(mail: &OutgoingMail) -> Result<Message, MailGatewayError> {
    let mut builder = Message::builder()
        .from(mailbox(&mail.from_name, &mail.from_address)?)
        .to(mailbox("", &mail.to)?)
        .subject(mail.subject.clone());
    if let Some(bcc) = &mail.bcc {
        builder = builder.bcc(mailbox("", bcc)?);
    }
    let text = SinglePart::plain(mail.body.clone());
    let result = match &mail.attachment {
        Some(attachment) => {
            let content_type = ContentType::parse(&attachment.content_type)
                .map_err(|e| MailGatewayError::InvalidMessage(e.to_string()))?;
            let part = Attachment::new(attachment.file_name.clone())
                .body(attachment.bytes.clone(), content_type);
            builder.multipart(MultiPart::mixed().singlepart(text).singlepart(part))
        }
        None => builder.singlepart(text),
    };
    result.map_err(|e| MailGatewayError::InvalidMessage(e.to_string()))
}

/// 530/534/535 are the refusals of a login; other 5xx refuse the message itself.
fn classify(error: lettre::transport::smtp::Error) -> MailGatewayError {
    if error.is_timeout() {
        return MailGatewayError::Timeout;
    }
    if let Some(code) = error.status() {
        let code = code.to_string();
        if matches!(code.as_str(), "530" | "534" | "535") {
            return MailGatewayError::Authentication;
        }
        return MailGatewayError::Rejected(format!("{error}"));
    }
    MailGatewayError::Connection(error.to_string())
}

#[cfg(test)]
#[path = "lettre_mail_gateway_test.rs"]
mod tests;
