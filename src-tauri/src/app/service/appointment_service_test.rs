use sea_orm::{ConnectionTrait, EntityTrait};

use super::*;
use crate::app::entity::recurrence_group;
use crate::app::model::appointment::AppointmentStatus;
use crate::test_support::{empty_database, FixedClock};

const NOW: &str = "2026-05-04 08:30:00";

async fn database_with_client() -> DatabaseConnection {
    let db = empty_database::setup().await;
    db.execute_unprepared(
        "INSERT INTO clients (id, first_name, last_name, fiscal_code)
           VALUES (1, 'Anna', 'Bianchi', 'RSSMRA80A41H501Y')",
    )
    .await
    .unwrap();
    db
}

#[tokio::test]
async fn a_new_appointment_is_stamped_by_the_clock() {
    let db = database_with_client().await;
    let input = CreateAppointmentInput {
        client_id: 1,
        service_id: None,
        date: "2026-05-05".to_string(),
        start_time: "10:00".to_string(),
        end_time: "11:00".to_string(),
        status: AppointmentStatus::Scheduled,
        notes: String::new(),
        recurrence_group_id: None,
    };

    let appointment = create(&db, &FixedClock::at(NOW), input).await.unwrap();

    assert_eq!(appointment.created_at, NOW);
    assert_eq!(appointment.updated_at, NOW);
}

#[tokio::test]
async fn a_recurrence_and_its_appointments_are_stamped_by_the_clock() {
    let db = database_with_client().await;
    let input = CreateRecurringAppointmentsInput {
        client_id: 1,
        service_id: None,
        dates: vec!["2026-05-05".to_string(), "2026-05-12".to_string()],
        start_time: "10:00".to_string(),
        end_time: "11:00".to_string(),
        notes: String::new(),
    };

    let created = create_recurring(&db, &FixedClock::at(NOW), input)
        .await
        .unwrap();

    assert!(created
        .iter()
        .all(|a| a.created_at == NOW && a.updated_at == NOW));
    let group = recurrence_group::Entity::find()
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(group.created_at, NOW);
}
