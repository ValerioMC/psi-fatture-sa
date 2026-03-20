use sea_orm::{ActiveValue::Set, DatabaseConnection};

use crate::app::entity::professional_config::{self, ActiveModel};
use crate::app::model::config::{ProfessionalConfig, TaxRegime, UpsertConfigInput};
use crate::app::repository::config_repository;

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
    let active = build_active_model(&input);
    let model = config_repository::save(db, active)
        .await
        .map_err(|e| e.to_string())?;
    Ok(into_domain(model))
}

// ─── Private helpers ──────────────────────────────────────────────────────────

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
        is_psicoanalista: Set(input.is_psicoanalista as i32),
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
        is_psicoanalista: m.is_psicoanalista != 0,
        created_at: m.created_at,
        updated_at: m.updated_at,
    }
}
