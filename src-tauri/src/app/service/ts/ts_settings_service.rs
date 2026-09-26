use sea_orm::{ActiveValue::Set, ConnectionTrait};

use crate::app::entity::ts_setting::ActiveModel;
use crate::app::model::ts::{TsEnvironment, TsSettings, UpdateTsSettingsInput};
use crate::app::repository::config_repository;
use crate::app::repository::ts::ts_settings_repository;
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
#[path = "ts_settings_service_test.rs"]
mod tests;
