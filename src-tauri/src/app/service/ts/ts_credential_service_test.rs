use super::*;
use crate::app::model::ts::TsEnvironment;
use crate::test_support::{InMemorySecretStore, UnavailableSecretStore, UnreadableSecretStore};

fn settings() -> TsSettings {
    TsSettings {
        environment: TsEnvironment::Test,
        username: "MTOMRA66A41G224M".to_string(),
        vat_number: "65498732105".to_string(),
    }
}

#[test]
fn reports_missing_secrets_on_empty_store() {
    let store = InMemorySecretStore::default();
    let status = status(&store).unwrap();
    assert!(!status.pincode_configured);
    assert!(!status.password_configured);
}

#[test]
fn saves_trimmed_pincode_and_untrimmed_password() {
    let store = InMemorySecretStore::default();
    save_pincode(&store, "  1234567890 ").unwrap();
    let result = save_password(&store, " Salve 123").unwrap();
    assert!(result.pincode_configured && result.password_configured);
    assert_eq!(
        store.read(SecretKind::TsPincode).unwrap().as_deref(),
        Some("1234567890")
    );
    assert_eq!(
        store.read(SecretKind::TsPassword).unwrap().as_deref(),
        Some(" Salve 123")
    );
}

#[test]
fn replaces_existing_pincode() {
    let store = InMemorySecretStore::default();
    save_pincode(&store, "1111").unwrap();
    save_pincode(&store, "2222").unwrap();
    assert_eq!(
        store.read(SecretKind::TsPincode).unwrap().as_deref(),
        Some("2222")
    );
}

#[test]
fn rejects_blank_spaced_or_oversized_secrets() {
    let store = InMemorySecretStore::default();
    assert!(save_pincode(&store, "   ").is_err());
    assert!(save_pincode(&store, "12 34").is_err());
    assert!(save_pincode(&store, &"9".repeat(SECRET_MAX_LENGTH + 1)).is_err());
    assert!(save_password(&store, "  ").is_err());
    assert!(!status(&store).unwrap().pincode_configured);
}

#[test]
fn delete_is_idempotent() {
    let store = InMemorySecretStore::default();
    save_pincode(&store, "1234").unwrap();
    assert!(!delete_pincode(&store).unwrap().pincode_configured);
    assert!(!delete_pincode(&store).unwrap().pincode_configured);
    assert!(!delete_password(&store).unwrap().password_configured);
}

#[test]
fn surfaces_store_failures() {
    let store = UnavailableSecretStore;
    assert!(status(&store)
        .unwrap_err()
        .to_string()
        .contains("Archivio credenziali"));
    assert!(save_pincode(&store, "1234").is_err());
    assert!(delete_password(&store).is_err());
}

#[test]
fn unreadable_secrets_read_as_missing_but_block_the_session() {
    let store = UnreadableSecretStore;
    let status = status(&store).unwrap();
    assert!(!status.password_configured && !status.pincode_configured);
    assert!(session(&store, &settings())
        .unwrap_err()
        .to_string()
        .contains("inseriscila di nuovo"));
}

#[test]
fn status_never_serializes_the_secrets() {
    let store = InMemorySecretStore::default();
    save_pincode(&store, "1234567890").unwrap();
    save_password(&store, "Salve123").unwrap();
    let json = serde_json::to_string(&status(&store).unwrap()).unwrap();
    assert!(!json.contains("1234567890") && !json.contains("Salve123"));
}

#[test]
fn session_names_the_first_missing_piece() {
    let store = InMemorySecretStore::default();
    assert!(session(&store, &settings())
        .unwrap_err()
        .to_string()
        .contains("password"));
    save_password(&store, "Salve123").unwrap();
    assert!(session(&store, &settings())
        .unwrap_err()
        .to_string()
        .contains("PINCODE"));
    save_pincode(&store, "3489543096").unwrap();

    let mut blank = settings();
    blank.vat_number.clear();
    assert!(session(&store, &blank)
        .unwrap_err()
        .to_string()
        .contains("partita IVA"));

    let ready = session(&store, &settings()).unwrap();
    assert_eq!(ready.username, "MTOMRA66A41G224M");
    assert_eq!(ready.environment, TsEnvironment::Test);
}
