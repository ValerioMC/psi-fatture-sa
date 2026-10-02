use super::*;
use crate::app::model::email::{EmailProvider, EmailSecurity, UpdateEmailAccountInput};
use crate::app::repository::email::MailGatewayError;
use crate::test_support::email_database::{self, CANCELLED, DRAFT, ISSUED, PAID};
use crate::test_support::{FakeMailGateway, InMemorySecretStore};

async fn configured(db: &sea_orm::DatabaseConnection, bcc_self: bool) -> InMemorySecretStore {
    email_account_service::update(
        db,
        UpdateEmailAccountInput {
            provider: EmailProvider::Psypec,
            sender_address: "maria.demo@psypec.it".into(),
            sender_name: "Dott.ssa Maria Demo".into(),
            smtp_host: String::new(),
            smtp_port: 465,
            security: EmailSecurity::Tls,
            username: String::new(),
            bcc_self,
        },
    )
    .await
    .unwrap();
    let store = InMemorySecretStore::default();
    email_credential_service::save_password(&store, "segreta").unwrap();
    store
}

fn input(invoice_id: i64, recipient: &str, remember: bool) -> SendInvoiceEmailInput {
    SendInvoiceEmailInput {
        invoice_id,
        recipient: recipient.into(),
        subject: "Fattura n. 12/2026".into(),
        body: "Gentile Anna Bianchi,\nin allegato la fattura.\n\n".into(),
        remember_recipient: remember,
    }
}

#[tokio::test]
async fn prepares_the_template_for_the_patient_on_file() {
    let db = email_database::setup(Some("anna@example.it")).await;
    let draft = prepare(&db, ISSUED).await.unwrap();
    assert_eq!(draft.recipient, "anna@example.it");
    assert!(draft.recipient_on_file);
    assert_eq!(draft.subject, "Fattura n. 12/2026 – Dott.ssa Maria Demo");
    assert!(draft.body.contains("fattura n. 12/2026 del 20 marzo 2026"));
    assert!(draft.body.contains("83,60 €"));
    assert_eq!(draft.attachment_name, "Fattura_12_2026.pdf");
}

#[tokio::test]
async fn drafts_and_cancelled_invoices_are_not_sent() {
    let db = email_database::setup(Some("anna@example.it")).await;
    assert!(prepare(&db, DRAFT)
        .await
        .unwrap_err()
        .to_string()
        .contains("Emetti"));
    assert!(prepare(&db, CANCELLED)
        .await
        .unwrap_err()
        .to_string()
        .contains("annullata"));
    let store = configured(&db, false).await;
    let gateway = FakeMailGateway::default();
    assert!(send(
        &db,
        &store,
        &gateway,
        input(DRAFT, "anna@example.it", false)
    )
    .await
    .is_err());
    assert!(gateway.sent().is_empty());
}

#[tokio::test]
async fn sends_the_pdf_from_the_saved_mailbox_and_records_it() {
    let db = email_database::setup(Some("anna@example.it")).await;
    let store = configured(&db, true).await;
    let gateway = FakeMailGateway::default();
    let record = send(
        &db,
        &store,
        &gateway,
        input(PAID, " Anna@Example.it ", false),
    )
    .await
    .unwrap();

    assert_eq!(record.status, InvoiceEmailStatus::Sent);
    assert_eq!(record.recipient, "anna@example.it");
    assert_eq!(record.attachment_name, "Fattura_13_2026.pdf");
    let sent = gateway.sent();
    assert_eq!(sent.len(), 1);
    let (session, mail) = &sent[0];
    assert_eq!(session.host, "smtps.sicurezzapostale.it");
    assert_eq!(session.password, "segreta");
    assert_eq!(mail.from_address, "maria.demo@psypec.it");
    assert_eq!(mail.bcc.as_deref(), Some("maria.demo@psypec.it"));
    assert!(
        mail.body.ends_with("fattura."),
        "trailing blank lines are trimmed"
    );
    let attachment = mail.attachment.as_ref().unwrap();
    assert!(attachment.bytes.starts_with(b"%PDF-"));
    assert_eq!(
        list(
            &db,
            InvoiceEmailFilters {
                invoice_id: Some(PAID)
            }
        )
        .await
        .unwrap(),
        vec![record]
    );
}

#[tokio::test]
async fn a_refused_send_is_recorded_and_reported() {
    let db = email_database::setup(Some("anna@example.it")).await;
    let store = configured(&db, false).await;
    let gateway = FakeMailGateway::failing(MailGatewayError::Authentication);
    let error = send(
        &db,
        &store,
        &gateway,
        input(ISSUED, "anna@example.it", false),
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("indirizzo e password"));
    let attempts = list(&db, InvoiceEmailFilters::default()).await.unwrap();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].status, InvoiceEmailStatus::Failed);
    assert_eq!(
        attempts[0].error.as_deref(),
        Some(error.to_string().as_str())
    );
}

#[tokio::test]
async fn nothing_is_sent_without_a_saved_mailbox_or_its_password() {
    let db = email_database::setup(Some("anna@example.it")).await;
    let gateway = FakeMailGateway::default();
    let empty = InMemorySecretStore::default();
    let error = send(
        &db,
        &empty,
        &gateway,
        input(ISSUED, "anna@example.it", false),
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("Configura la casella"));
    configured(&db, false).await;
    let error = send(
        &db,
        &empty,
        &gateway,
        input(ISSUED, "anna@example.it", false),
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("password"));
    assert!(gateway.sent().is_empty());
    assert!(list(&db, InvoiceEmailFilters::default())
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn refuses_a_malformed_recipient_before_calling_the_server() {
    let db = email_database::setup(None).await;
    let store = configured(&db, false).await;
    let gateway = FakeMailGateway::default();
    assert!(send(
        &db,
        &store,
        &gateway,
        input(ISSUED, "anna.example.it", false)
    )
    .await
    .is_err());
    assert!(send(&db, &store, &gateway, input(ISSUED, "", false))
        .await
        .is_err());
    assert!(gateway.sent().is_empty());
}

#[tokio::test]
async fn remembers_a_new_address_on_the_patient_only_after_a_send() {
    let db = email_database::setup(None).await;
    let store = configured(&db, false).await;
    let failing = FakeMailGateway::failing(MailGatewayError::Timeout);
    assert!(send(
        &db,
        &store,
        &failing,
        input(ISSUED, "anna@example.it", true)
    )
    .await
    .is_err());
    assert!(!prepare(&db, ISSUED).await.unwrap().recipient_on_file);

    send(
        &db,
        &store,
        &FakeMailGateway::default(),
        input(ISSUED, "anna@example.it", true),
    )
    .await
    .unwrap();
    let draft = prepare(&db, PAID).await.unwrap();
    assert_eq!(draft.recipient, "anna@example.it");
}

#[tokio::test]
async fn a_bulk_send_skips_patients_without_an_address() {
    let db = email_database::setup(None).await;
    let store = configured(&db, false).await;
    let gateway = FakeMailGateway::default();
    let error = send_prepared(&db, &store, &gateway, ISSUED)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("indirizzo email"));
    assert!(gateway.sent().is_empty());
}

#[tokio::test]
async fn checks_the_login_and_says_where_invoices_leave_from() {
    let db = email_database::setup(None).await;
    let store = configured(&db, false).await;
    let check = check_connection(&db, &store, &FakeMailGateway::default())
        .await
        .unwrap();
    assert!(check.ok && check.message.contains("maria.demo@psypec.it"));
    let failing = FakeMailGateway::failing(MailGatewayError::Connection("rete assente".into()));
    let check = check_connection(&db, &store, &failing).await.unwrap();
    assert!(!check.ok && check.message.contains("rete assente"));
}
