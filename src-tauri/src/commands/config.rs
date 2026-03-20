/// Tauri commands for professional config management.
use crate::db;
use crate::domain::models::{ProfessionalConfig, TaxRegime, UpsertConfigInput};
use rusqlite::params;

#[tauri::command]
pub fn get_config() -> Result<Option<ProfessionalConfig>, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    let result = conn.query_row(
        "SELECT id, title, first_name, last_name, vat_number, fiscal_code, tax_regime,
                albo_number, albo_region, address, city, province, zip_code, country,
                phone, pec_email, iban, coefficient, is_psicoanalista, created_at, updated_at
         FROM professional_config WHERE id = 1",
        [],
        row_to_config,
    );

    match result {
        Ok(config) => Ok(Some(config)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn upsert_config(input: UpsertConfigInput) -> Result<ProfessionalConfig, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO professional_config
            (id, title, first_name, last_name, vat_number, fiscal_code, tax_regime,
             albo_number, albo_region, address, city, province, zip_code, country,
             phone, pec_email, iban, coefficient, is_psicoanalista, updated_at)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, datetime('now'))
         ON CONFLICT(id) DO UPDATE SET
             title = excluded.title,
             first_name = excluded.first_name,
             last_name = excluded.last_name,
             vat_number = excluded.vat_number,
             fiscal_code = excluded.fiscal_code,
             tax_regime = excluded.tax_regime,
             albo_number = excluded.albo_number,
             albo_region = excluded.albo_region,
             address = excluded.address,
             city = excluded.city,
             province = excluded.province,
             zip_code = excluded.zip_code,
             country = excluded.country,
             phone = excluded.phone,
             pec_email = excluded.pec_email,
             iban = excluded.iban,
             coefficient = excluded.coefficient,
             is_psicoanalista = excluded.is_psicoanalista,
             updated_at = datetime('now')",
        params![
            input.title, input.first_name, input.last_name,
            input.vat_number, input.fiscal_code, input.tax_regime.as_str(),
            input.albo_number, input.albo_region, input.address,
            input.city, input.province, input.zip_code, input.country,
            input.phone, input.pec_email, input.iban,
            input.coefficient, input.is_psicoanalista as i64
        ],
    ).map_err(|e| e.to_string())?;

    get_config()?.ok_or_else(|| "Config not found after upsert".to_string())
}

fn row_to_config(row: &rusqlite::Row) -> rusqlite::Result<ProfessionalConfig> {
    Ok(ProfessionalConfig {
        id: row.get(0)?,
        title: row.get(1)?,
        first_name: row.get(2)?,
        last_name: row.get(3)?,
        vat_number: row.get(4)?,
        fiscal_code: row.get(5)?,
        tax_regime: TaxRegime::from(row.get::<_, String>(6)?),
        albo_number: row.get(7)?,
        albo_region: row.get(8)?,
        address: row.get(9)?,
        city: row.get(10)?,
        province: row.get(11)?,
        zip_code: row.get(12)?,
        country: row.get(13)?,
        phone: row.get(14)?,
        pec_email: row.get(15)?,
        iban: row.get(16)?,
        coefficient: row.get(17)?,
        is_psicoanalista: row.get::<_, i64>(18)? != 0,
        created_at: row.get(19)?,
        updated_at: row.get(20)?,
    })
}
