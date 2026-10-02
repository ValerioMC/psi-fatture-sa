use super::*;
use crate::app::model::email::{EmailProvider, EmailSecurity};
use crate::test_support::{InMemorySecretStore, UnreadableSecretStore};

fn account() -> EmailAccount {
    EmailAccount {
        provider: EmailProvider::Psypec,
        sender_address: "maria.demo@psypec.it".into(),
        sender_name: "Dott.ssa Maria Demo".into(),
        smtp_host: "smtps.sicurezzapostale.it".into(),
        smtp_port: 465,
        security: EmailSecurity::Tls,
        username: "maria.demo@psypec.it".into(),
        bcc_self: false,
        saved: true,
    }
}

#[test]
fn stores_the_password_as_typed_and_reports_it() {
    let store = InMemorySecretStore::default();
    assert!(!status(&store).unwrap().password_configured);
    assert!(
        save_password(&store, " segreta 1 ")
            .unwrap()
            .password_configured
    );
    assert_eq!(session(&store, &account()).unwrap().password, " segreta 1 ");
    assert!(!delete_password(&store).unwrap().password_configured);
}

#[test]
fn refuses_blank_or_oversized_passwords() {
    let store = InMemorySecretStore::default();
    assert!(save_password(&store, "   ").is_err());
    assert!(save_password(&store, &"x".repeat(PASSWORD_MAX_LENGTH + 1)).is_err());
}

#[test]
fn a_session_needs_the_password_and_carries_the_server() {
    let store = InMemorySecretStore::default();
    assert!(session(&store, &account())
        .unwrap_err()
        .to_string()
        .contains("password"));
    save_password(&store, "segreta").unwrap();
    let session = session(&store, &account()).unwrap();
    assert_eq!(session.host, "smtps.sicurezzapostale.it");
    assert_eq!(session.username, "maria.demo@psypec.it");
    assert!(
        !format!("{session:?}").contains("segreta"),
        "Debug never shows the password"
    );
}

#[test]
fn an_unreadable_password_reads_as_missing() {
    assert!(!status(&UnreadableSecretStore).unwrap().password_configured);
}
