use sea_orm::{ActiveValue::Set, DatabaseConnection};

use crate::app::entity::professional_config::{self, ActiveModel};
use crate::app::model::config::{Profession, ProfessionalConfig, TaxRegime, UpsertConfigInput};
use crate::app::repository::config_repository;
use crate::app::service::validation_service as validate;

/// Returns the professional config, or None if not yet configured.
pub async fn get(db: &DatabaseConnection) -> Result<Option<ProfessionalConfig>, String> {
    let model = config_repository::find(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(model.map(into_domain))
}

/// Inserts or updates the professional config (singleton, id=1).
pub async fn upsert(
    db: &DatabaseConnection,
    input: UpsertConfigInput,
) -> Result<ProfessionalConfig, String> {
    validate_config_input(&input)?;
    let active = build_active_model(&input);
    let model = config_repository::save(db, active)
        .await
        .map_err(|e| e.to_string())?;
    Ok(into_domain(model))
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn validate_config_input(input: &UpsertConfigInput) -> Result<(), String> {
    validate::validate_required(&input.first_name, "Nome")?;
    validate::validate_required(&input.last_name, "Cognome")?;
    validate::validate_required(&input.vat_number, "Partita IVA")?;
    validate::validate_required(&input.fiscal_code, "Codice fiscale")?;
    validate::validate_required(&input.address, "Indirizzo")?;
    validate::validate_required(&input.city, "Città")?;
    validate::validate_required(&input.province, "Provincia")?;
    validate::validate_required(&input.zip_code, "CAP")?;
    validate::validate_vat_number(&input.vat_number)?;
    validate::validate_fiscal_code(&input.fiscal_code)?;
    if !input.coefficient.is_finite() || !(1.0..=100.0).contains(&input.coefficient) {
        return Err("Coefficiente non valido (1-100)".to_string());
    }
    if input.initial_invoice_number < 1 {
        return Err("Il numero iniziale delle fatture deve essere almeno 1".to_string());
    }
    Ok(())
}

fn build_active_model(input: &UpsertConfigInput) -> ActiveModel {
    ActiveModel {
        id: Set(1),
        title: Set(input.title.clone()),
        first_name: Set(input.first_name.clone()),
        last_name: Set(input.last_name.clone()),
        vat_number: Set(input.vat_number.clone()),
        fiscal_code: Set(input.fiscal_code.clone()),
        tax_regime: Set(input.tax_regime.as_str().to_owned()),
        albo_number: Set(input.albo_number.clone()),
        albo_region: Set(input.albo_region.clone()),
        address: Set(input.address.clone()),
        city: Set(input.city.clone()),
        province: Set(input.province.clone()),
        zip_code: Set(input.zip_code.clone()),
        country: Set(input.country.clone()),
        phone: Set(input.phone.clone()),
        pec_email: Set(input.pec_email.clone()),
        iban: Set(input.iban.clone()),
        coefficient: Set(input.coefficient),
        profession: Set(input.profession.as_str().to_owned()),
        is_psicoanalista: Set(input.is_psicoanalista as i32),
        initial_invoice_number: Set(input.initial_invoice_number),
        updated_at: Set(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        ..Default::default()
    }
}

fn into_domain(m: professional_config::Model) -> ProfessionalConfig {
    ProfessionalConfig {
        id: m.id,
        title: m.title,
        first_name: m.first_name,
        last_name: m.last_name,
        vat_number: m.vat_number,
        fiscal_code: m.fiscal_code,
        tax_regime: TaxRegime::from(m.tax_regime),
        albo_number: m.albo_number,
        albo_region: m.albo_region,
        address: m.address,
        city: m.city,
        province: m.province,
        zip_code: m.zip_code,
        country: m.country,
        phone: m.phone,
        pec_email: m.pec_email,
        iban: m.iban,
        coefficient: m.coefficient,
        profession: Profession::from(m.profession),
        is_psicoanalista: m.is_psicoanalista != 0,
        initial_invoice_number: m.initial_invoice_number,
        created_at: m.created_at,
        updated_at: m.updated_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> UpsertConfigInput {
        UpsertConfigInput {
            title: "Dott.".to_string(),
            first_name: "Maria".to_string(),
            last_name: "Rossi".to_string(),
            vat_number: "12345678903".to_string(),
            fiscal_code: "RSSMRA80A41H501Y".to_string(),
            tax_regime: TaxRegime::Forfettario,
            albo_number: "1234".to_string(),
            albo_region: "Lazio".to_string(),
            address: "Via Roma 1".to_string(),
            city: "Roma".to_string(),
            province: "RM".to_string(),
            zip_code: "00100".to_string(),
            country: "IT".to_string(),
            phone: String::new(),
            pec_email: String::new(),
            iban: String::new(),
            coefficient: 78.0,
            profession: Profession::Psicoterapeuta,
            is_psicoanalista: true,
            initial_invoice_number: 1,
        }
    }

    #[test]
    fn accepts_valid_input() {
        assert!(validate_config_input(&valid_input()).is_ok());
    }

    #[test]
    fn rejects_blank_required_fields() {
        for field in [
            "first_name",
            "last_name",
            "vat_number",
            "fiscal_code",
            "address",
            "city",
            "province",
            "zip_code",
        ] {
            let mut input = valid_input();
            match field {
                "first_name" => input.first_name = "  ".to_string(),
                "last_name" => input.last_name = String::new(),
                "vat_number" => input.vat_number = String::new(),
                "fiscal_code" => input.fiscal_code = String::new(),
                "address" => input.address = String::new(),
                "city" => input.city = String::new(),
                "province" => input.province = String::new(),
                _ => input.zip_code = String::new(),
            }
            let result = validate_config_input(&input);
            assert!(result.is_err(), "expected error for blank {field}");
            assert!(result.unwrap_err().contains("campo obbligatorio"));
        }
    }

    #[test]
    fn rejects_malformed_fiscal_data() {
        let mut bad_vat = valid_input();
        bad_vat.vat_number = "123".to_string();
        assert!(validate_config_input(&bad_vat).is_err());

        let mut bad_cf = valid_input();
        bad_cf.fiscal_code = "NOTVALID".to_string();
        assert!(validate_config_input(&bad_cf).is_err());
    }

    #[test]
    fn rejects_out_of_range_numbers() {
        let mut bad_coefficient = valid_input();
        bad_coefficient.coefficient = 0.0;
        assert!(validate_config_input(&bad_coefficient).is_err());

        let mut bad_start = valid_input();
        bad_start.initial_invoice_number = 0;
        assert!(validate_config_input(&bad_start).is_err());
    }

    /// End-to-end registration through the same boundary the frontend uses:
    /// the exact JSON payload sent by `upsertConfig` in `src/api.ts` is
    /// deserialized, saved on a migrated in-memory SQLite and read back.
    #[tokio::test]
    async fn registration_persists_profession_and_psicoanalista_flag() {
        use sea_orm::Database;
        use sea_orm_migration::MigratorTrait;

        let db = Database::connect("sqlite::memory:").await.unwrap();
        crate::migration::Migrator::up(&db, None).await.unwrap();

        let payload = r#"{
            "title": "Dott.ssa", "first_name": "Maria", "last_name": "Rossi",
            "vat_number": "12345678903", "fiscal_code": "RSSMRA80A41H501Y",
            "tax_regime": "forfettario", "albo_number": "1234", "albo_region": "Lazio",
            "address": "Via Roma 1", "city": "Roma", "province": "RM",
            "zip_code": "00100", "country": "IT", "phone": "",
            "pec_email": "mario.rossi@pec.it", "iban": "IT60X0542811101000000123456",
            "coefficient": 78, "profession": "psicoterapeuta",
            "is_psicoanalista": true, "initial_invoice_number": 1
        }"#;
        let input: UpsertConfigInput = serde_json::from_str(payload).unwrap();

        let saved = upsert(&db, input).await.unwrap();
        assert_eq!(saved.profession, Profession::Psicoterapeuta);
        assert!(saved.is_psicoanalista);

        let reloaded = get(&db).await.unwrap().unwrap();
        assert_eq!(reloaded.profession, Profession::Psicoterapeuta);
        assert!(reloaded.is_psicoanalista);
        assert_eq!(
            serde_json::to_value(&reloaded.profession).unwrap(),
            serde_json::json!("psicoterapeuta")
        );
    }

    #[tokio::test]
    async fn registration_rejects_invalid_fiscal_code_with_clear_message() {
        use sea_orm::Database;
        use sea_orm_migration::MigratorTrait;

        let db = Database::connect("sqlite::memory:").await.unwrap();
        crate::migration::Migrator::up(&db, None).await.unwrap();

        let mut input = valid_input();
        input.fiscal_code = "RSSMRA80A41H501X".to_string();
        let err = upsert(&db, input).await.unwrap_err();
        assert!(err.contains("carattere di controllo errato"));

        assert!(get(&db).await.unwrap().is_none());
    }

    #[test]
    fn profession_round_trips_through_storage_string() {
        assert_eq!(
            Profession::from("psicologo".to_string()),
            Profession::Psicologo
        );
        assert_eq!(
            Profession::from("psicoterapeuta".to_string()),
            Profession::Psicoterapeuta
        );
        assert_eq!(
            Profession::from("unknown".to_string()),
            Profession::Psicologo
        );
        assert_eq!(Profession::Psicoterapeuta.as_str(), "psicoterapeuta");
    }
}
