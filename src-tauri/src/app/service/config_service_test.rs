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
        specialization: String::new(),
        hide_quantity_in_invoice: false,
        enpap_excludes_bollo: false,
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("campo obbligatorio"));
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
        "is_psicoanalista": true, "specialization": "", "hide_quantity_in_invoice": false,
        "initial_invoice_number": 1
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
    assert!(err.to_string().contains("carattere di controllo errato"));

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
