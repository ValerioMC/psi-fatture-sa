use super::*;
use crate::app::repository::email::MailAttachment;

fn mail(attachment: Option<MailAttachment>) -> OutgoingMail {
    OutgoingMail {
        from_address: "maria.demo@psypec.it".into(),
        from_name: "Dott.ssa Maria Demo".into(),
        to: "anna@example.it".into(),
        bcc: Some("maria.demo@psypec.it".into()),
        subject: "Fattura n. 12/2026 – Dott.ssa Maria Demo".into(),
        body: "Gentile Anna Bianchi,\nin allegato la fattura.".into(),
        attachment,
    }
}

#[test]
fn builds_a_multipart_message_with_the_pdf() {
    let pdf = MailAttachment {
        file_name: "Fattura_12_2026.pdf".into(),
        content_type: "application/pdf".into(),
        bytes: b"%PDF-1.7 test".to_vec(),
    };
    let raw = String::from_utf8(build_message(&mail(Some(pdf))).unwrap().formatted()).unwrap();
    assert!(raw.contains("multipart/mixed"));
    assert!(raw.contains("application/pdf"));
    assert!(raw.contains("filename=\"Fattura_12_2026.pdf\""));
    assert!(raw.contains("To: anna@example.it"));
    assert!(
        !raw.contains("Bcc:"),
        "the blind copy never shows in the headers"
    );
}

#[test]
fn builds_a_plain_message_without_attachment() {
    let raw = String::from_utf8(build_message(&mail(None)).unwrap().formatted()).unwrap();
    assert!(raw.contains("text/plain; charset=utf-8"));
    assert!(!raw.contains("multipart"));
}

#[test]
fn refuses_a_malformed_recipient() {
    let mut broken = mail(None);
    broken.to = "anna.example.it".into();
    assert!(matches!(
        build_message(&broken),
        Err(MailGatewayError::InvalidMessage(_))
    ));
}

fn psypec_session() -> SmtpSession {
    SmtpSession {
        host: "smtps.sicurezzapostale.it".into(),
        port: 465,
        security: EmailSecurity::Tls,
        username: "nessuno@psypec.it".into(),
        password: "password-sbagliata".into(),
    }
}

/// Reaches the real psypec.it server: TLS must verify and a wrong password must read
/// as a refused login, not as a network problem.
#[tokio::test]
#[ignore]
async fn live_psypec_refuses_a_wrong_password() {
    let outcome = LettreMailGateway.check(&psypec_session()).await;
    assert_eq!(outcome, Err(MailGatewayError::Authentication));
}
