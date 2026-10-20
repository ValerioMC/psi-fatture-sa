use super::*;
use crate::test_support::{empty_database, FixedClock};

fn client_input() -> CreateClientInput {
    CreateClientInput {
        client_type: ClientType::PersonaFisica,
        first_name: "Anna".to_string(),
        last_name: "Bianchi".to_string(),
        birth_date: None,
        gender: None,
        fiscal_code: "RSSMRA80A41H501Y".to_string(),
        vat_number: None,
        address: "Via Roma 1".to_string(),
        city: "Roma".to_string(),
        province: "RM".to_string(),
        zip_code: "00100".to_string(),
        email: None,
        phone: String::new(),
        notes: None,
        sts_authorization: true,
        hide_quantity_in_invoice: None,
    }
}

#[tokio::test]
async fn a_new_client_is_stamped_by_the_clock() {
    let db = empty_database::setup().await;
    let clock = FixedClock::at("2026-05-04 08:30:00");

    let client = create(&db, &clock, client_input()).await.unwrap();

    assert_eq!(client.created_at, "2026-05-04 08:30:00");
    assert_eq!(client.updated_at, "2026-05-04 08:30:00");
}
