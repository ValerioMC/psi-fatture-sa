use sea_orm::{ActiveValue::Set, ConnectionTrait};

use crate::app::entity::ts_setting::ActiveModel;
use crate::app::model::ts::{TsEnvironment, TsSettings, UpdateTsSettingsInput};
use crate::app::repository::{config_repository, ts_settings_repository};
use crate::app::service::validation_service as validate;

/// The Sistema TS settings. Until saved, or when saved for an environment
/// this build does not have, they point at production with the codice fiscale
/// and P.IVA of the professional profile.
pub async fn get(db: &impl ConnectionTrait) -> Result<TsSettings, String> {
    get_within(db, TsEnvironment::available()).await
}

/// Refuses an environment this build does not have: a distributed release
/// never saves the test one.
pub async fn update(
    db: &impl ConnectionTrait,
    input: UpdateTsSettingsInput,
) -> Result<TsSettings, String> {
    update_within(db, input, TsEnvironment::available()).await
}

async fn get_within(
    db: &impl ConnectionTrait,
    available: &[TsEnvironment],
) -> Result<TsSettings, String> {
    if let Some(row) = ts_settings_repository::find(db).await? {
        let environment = TsEnvironment::parse(&row.environment)?;
        if available.contains(&environment) {
            return Ok(TsSettings {
                environment,
                username: row.username,
                vat_number: row.vat_number,
            });
        }
        log::warn!(target: "sistema_ts", environment = environment.as_str(); "saved environment not in this build, falling back to production");
    }
    let profile = config_repository::find(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(TsSettings {
        environment: TsEnvironment::Produzione,
        username: profile
            .as_ref()
            .map(|p| p.fiscal_code.clone())
            .unwrap_or_default(),
        vat_number: profile.map(|p| p.vat_number).unwrap_or_default(),
    })
}

async fn update_within(
    db: &impl ConnectionTrait,
    input: UpdateTsSettingsInput,
    available: &[TsEnvironment],
) -> Result<TsSettings, String> {
    if !available.contains(&input.environment) {
        return Err(
            "L'ambiente di test Sogei non è disponibile in questa versione dell'app".to_string(),
        );
    }
    let username = input.username.trim().to_uppercase();
    let vat_number = input.vat_number.trim().to_string();
    validate_identity(input.environment, &username, &vat_number)?;

    let active = ActiveModel {
        id: Set(1),
        environment: Set(input.environment.as_str().to_owned()),
        username: Set(username),
        vat_number: Set(vat_number),
        updated_at: Set(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
    };
    ts_settings_repository::save(db, active).await?;
    get_within(db, available).await
}

/// Production data must be real; the test kit's P.IVA fails the checksum,
/// so the test environment only checks the shape.
fn validate_identity(
    environment: TsEnvironment,
    username: &str,
    vat_number: &str,
) -> Result<(), String> {
    validate::validate_required(username, "Codice fiscale Sistema TS")?;
    validate::validate_required(vat_number, "Partita IVA Sistema TS")?;
    if username.chars().count() != 16 || !username.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("Il codice fiscale di accesso deve avere 16 caratteri".to_string());
    }
    if vat_number.len() != 11 || !vat_number.chars().all(|c| c.is_ascii_digit()) {
        return Err("La partita IVA deve essere composta da 11 cifre".to_string());
    }
    if environment == TsEnvironment::Produzione {
        validate::validate_fiscal_code(username)?;
        validate::validate_vat_number(vat_number)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
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
    ) -> Result<TsSettings, String> {
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
        assert!(!err.is_empty());
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
        assert!(err.contains("non è disponibile"));
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
}
