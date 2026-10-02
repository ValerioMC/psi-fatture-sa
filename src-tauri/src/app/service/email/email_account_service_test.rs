use super::*;
use crate::test_support::email_database;

fn input(provider: EmailProvider, address: &str) -> UpdateEmailAccountInput {
    UpdateEmailAccountInput {
        provider,
        sender_address: address.to_string(),
        sender_name: " Dott.ssa Maria Demo ".to_string(),
        smtp_host: "mail.example.it".to_string(),
        smtp_port: 587,
        security: EmailSecurity::Starttls,
        username: String::new(),
        bcc_self: true,
    }
}

#[tokio::test]
async fn proposes_the_profile_pec_on_psypec_before_the_first_save() {
    let db = email_database::setup(None).await;
    let account = get(&db).await.unwrap();
    assert!(!account.saved);
    assert_eq!(account.provider, EmailProvider::Psypec);
    assert_eq!(account.sender_address, "maria.demo@psypec.it");
    assert_eq!(account.username, "maria.demo@psypec.it");
    assert_eq!(account.sender_name, "Dott.ssa Maria Demo");
    assert_eq!(account.smtp_host, "smtps.sicurezzapostale.it");
    assert_eq!(
        (account.smtp_port, account.security),
        (465, EmailSecurity::Tls)
    );
    assert!(saved(&db)
        .await
        .unwrap_err()
        .to_string()
        .contains("Impostazioni"));
}

#[tokio::test]
async fn a_preset_provider_keeps_the_preset_server_whatever_is_typed() {
    let db = email_database::setup(None).await;
    let account = update(&db, input(EmailProvider::Psypec, " Maria.Demo@Psypec.it "))
        .await
        .unwrap();
    assert!(account.saved);
    assert_eq!(account.sender_address, "maria.demo@psypec.it");
    assert_eq!(
        account.username, "maria.demo@psypec.it",
        "blank username is the address"
    );
    assert_eq!(account.sender_name, "Dott.ssa Maria Demo");
    assert_eq!(account.smtp_host, "smtps.sicurezzapostale.it");
    assert_eq!(
        (account.smtp_port, account.security),
        (465, EmailSecurity::Tls)
    );
    assert!(account.bcc_self);
    assert_eq!(saved(&db).await.unwrap(), account);
}

#[tokio::test]
async fn a_custom_provider_stores_the_server_as_typed() {
    let db = email_database::setup(None).await;
    let mut custom = input(EmailProvider::Custom, "studio@example.it");
    custom.username = "studio".to_string();
    let account = update(&db, custom).await.unwrap();
    assert_eq!(account.smtp_host, "mail.example.it");
    assert_eq!(
        (account.smtp_port, account.security),
        (587, EmailSecurity::Starttls)
    );
    assert_eq!(account.username, "studio");
}

#[tokio::test]
async fn refuses_malformed_accounts() {
    let db = email_database::setup(None).await;
    assert!(update(&db, input(EmailProvider::Psypec, "")).await.is_err());
    assert!(update(&db, input(EmailProvider::Psypec, "maria.psypec.it"))
        .await
        .is_err());
    let mut host = input(EmailProvider::Custom, "studio@example.it");
    host.smtp_host = "mail example".to_string();
    assert!(update(&db, host).await.is_err());
    let mut port = input(EmailProvider::Custom, "studio@example.it");
    port.smtp_port = 0;
    assert!(update(&db, port).await.is_err());
    assert!(!get(&db).await.unwrap().saved);
}

#[test]
fn detects_the_provider_from_the_domain() {
    assert_eq!(
        EmailProvider::detect("maria@PSYPEC.it"),
        EmailProvider::Psypec
    );
    assert_eq!(
        EmailProvider::detect("maria@pec.it"),
        EmailProvider::ArubaPec
    );
    assert_eq!(
        EmailProvider::detect("maria@gmail.com"),
        EmailProvider::Gmail
    );
    assert_eq!(
        EmailProvider::detect("maria@studio.it"),
        EmailProvider::Custom
    );
    assert_eq!(
        EmailProvider::detect("senza-chiocciola"),
        EmailProvider::Custom
    );
}

#[test]
fn offers_every_provider_with_psypec_first() {
    let presets = providers();
    assert_eq!(presets.len(), EmailProvider::ALL.len());
    assert_eq!(presets[0].provider, EmailProvider::Psypec);
    assert!(presets[0].certified);
}
