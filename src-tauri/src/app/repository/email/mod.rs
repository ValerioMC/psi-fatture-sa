//! The professional's mailbox: its settings, the template, the log of invoices
//! emailed, and the SMTP gateway that sends them.

pub mod email_account_repository;
pub mod email_template_repository;
pub mod invoice_email_repository;
pub mod lettre_mail_gateway;
pub mod mail_attachment;
pub mod mail_gateway;
pub mod mail_gateway_error;
pub mod outgoing_mail;
pub mod smtp_session;

pub use lettre_mail_gateway::LettreMailGateway;
pub use mail_attachment::MailAttachment;
pub use mail_gateway::MailGateway;
pub use mail_gateway_error::MailGatewayError;
pub use outgoing_mail::OutgoingMail;
pub use smtp_session::SmtpSession;
