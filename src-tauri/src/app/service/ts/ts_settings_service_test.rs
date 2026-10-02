use super::*;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

async fn setup() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    crate::migration::Migrator::up(&db, None).await.unwrap();
    db
}

const PRODUCTION_ONLY: &[TsEnvironment] = &[TsEnvironment::Produzione];
const EVERY_ENVIRONMENT: &[TsEnvironment] = &[TsEnvironment::Produzione, TsEnvironment::Test];

/// Saves as a developer build does, whatever build runs the tests.
async fn save_anywhere(
    db: &DatabaseConnection,
    input: UpdateTsSettingsInput,
) -> Result<TsSettings, AppError> {
    update_within(db, input, EVERY_ENVIRONMENT).await
}

fn input(environment: TsEnvironment, username: &str, vat: &str) -> UpdateTsSettingsInput {
    UpdateTsSettingsInput {
        environment,
        username: username.to_string(),
        vat_number: vat.to_string(),
    }
}

#[tokio::test]
async fn defaults_to_production_with_the_profile_identity() {
    let db = setup().await;
    db.execute_unprepared(
        "INSERT INTO professional_config (id, fiscal_code, vat_number) VALUES (1, 'RSSMRA80A41H501Y', '12345678903')",
    )
    .await
    .unwrap();
    let settings = get(&db).await.unwrap();
    assert_eq!(settings.environment, TsEnvironment::Produzione);
    assert_eq!(settings.username, "RSSMRA80A41H501Y");
    assert_eq!(settings.vat_number, "12345678903");
}

#[tokio::test]
async fn saves_the_test_kit_identity_despite_its_checksums() {
    let db = setup().await;
    let saved = save_anywhere(
        &db,
        input(TsEnvironment::Test, " mtomra66a41g224m ", "65498732105"),
    )
    .await
    .unwrap();
    assert_eq!(saved.environment, TsEnvironment::Test);
    assert_eq!(saved.username, "MTOMRA66A41G224M");
    assert_eq!(get_within(&db, EVERY_ENVIRONMENT).await.unwrap(), saved);
}

#[tokio::test]
async fn production_requires_valid_checksums() {
    let db = setup().await;
    let err = update(
        &db,
        input(TsEnvironment::Produzione, "MTOMRA66A41G224M", "65498732105"),
    )
    .await
    .unwrap_err();
    assert!(!err.to_string().is_empty());
    assert!(update(
        &db,
        input(TsEnvironment::Produzione, "RSSMRA80A41H501Y", "12345678903")
    )
    .await
    .is_ok());
}

#[tokio::test]
async fn a_production_only_build_refuses_the_test_environment() {
    let db = setup().await;
    let err = update_within(
        &db,
        input(TsEnvironment::Test, "MTOMRA66A41G224M", "65498732105"),
        PRODUCTION_ONLY,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("non è disponibile"));
    assert!(ts_settings_repository::find(&db).await.unwrap().is_none());
}

#[tokio::test]
async fn a_production_only_build_ignores_saved_test_settings() {
    let db = setup().await;
    db.execute_unprepared(
        "INSERT INTO professional_config (id, fiscal_code, vat_number) VALUES (1, 'RSSMRA80A41H501Y', '12345678903')",
    )
    .await
    .unwrap();
    save_anywhere(
        &db,
        input(TsEnvironment::Test, "MTOMRA66A41G224M", "65498732105"),
    )
    .await
    .unwrap();
    let settings = get_within(&db, PRODUCTION_ONLY).await.unwrap();
    assert_eq!(settings.environment, TsEnvironment::Produzione);
    assert_eq!(settings.username, "RSSMRA80A41H501Y");
    assert_eq!(settings.vat_number, "12345678903");
}

#[tokio::test]
async fn rejects_malformed_identity() {
    let db = setup().await;
    assert!(
        save_anywhere(&db, input(TsEnvironment::Test, "SHORT", "65498732105"))
            .await
            .is_err()
    );
    assert!(
        save_anywhere(&db, input(TsEnvironment::Test, "MTOMRA66A41G224M", "123"))
            .await
            .is_err()
    );
    assert!(save_anywhere(&db, input(TsEnvironment::Test, "", ""))
        .await
        .is_err());
}
