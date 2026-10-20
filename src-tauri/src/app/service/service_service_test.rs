use super::*;
use crate::test_support::{empty_database, FixedClock};

#[tokio::test]
async fn a_new_service_is_stamped_by_the_clock() {
    let db = empty_database::setup().await;
    let clock = FixedClock::at("2026-05-04 08:30:00");
    let input = CreateServiceInput {
        name: "Colloquio".to_string(),
        description: String::new(),
        default_price: 70.0,
        vat_rate: 0.0,
        is_active: true,
    };

    let service = create(&db, &clock, input).await.unwrap();

    assert_eq!(service.created_at, "2026-05-04 08:30:00");
    assert_eq!(service.updated_at, "2026-05-04 08:30:00");
}
