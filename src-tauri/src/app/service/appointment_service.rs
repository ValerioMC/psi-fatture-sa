use sea_orm::{ActiveValue::Set, DatabaseConnection, TransactionTrait};

use crate::app::entity::appointment as appointments;
use crate::app::model::appointment::{
    Appointment, CreateAppointmentInput, CreateRecurringAppointmentsInput, UpdateAppointmentInput,
};
use crate::app::repository::appointment_repository;

/// Lists appointments with optional date range and client filters.
pub async fn list(
    db: &DatabaseConnection,
    date_from: Option<String>,
    date_to: Option<String>,
    client_id: Option<i64>,
) -> Result<Vec<Appointment>, String> {
    appointment_repository::find_all(db, date_from, date_to, client_id).await
}

/// Returns a single appointment by id.
pub async fn get(db: &DatabaseConnection, id: i64) -> Result<Appointment, String> {
    appointment_repository::find_by_id(db, id).await
}

/// Creates a single appointment and returns it.
pub async fn create(
    db: &DatabaseConnection,
    input: CreateAppointmentInput,
) -> Result<Appointment, String> {
    let active = build_active(
        input.client_id,
        input.service_id,
        &input.date,
        &input.start_time,
        &input.end_time,
        input.status.as_str(),
        Some(input.notes),
        input.recurrence_group_id,
    );

    let model = appointment_repository::insert(db, active)
        .await
        .map_err(|e| e.to_string())?;
    appointment_repository::find_by_id(db, model.id).await
}

/// Creates multiple recurring appointments in a transaction.
pub async fn create_recurring(
    db: &DatabaseConnection,
    input: CreateRecurringAppointmentsInput,
) -> Result<Vec<Appointment>, String> {
    let tx = db.begin().await.map_err(|e| e.to_string())?;

    let group_id = appointment_repository::insert_recurrence_group(&tx, input.client_id)
        .await
        .map_err(|e| e.to_string())?;

    let mut ids = Vec::new();
    for date in &input.dates {
        let active = build_active(
            input.client_id,
            input.service_id,
            date,
            &input.start_time,
            &input.end_time,
            "scheduled",
            Some(input.notes.clone()),
            Some(group_id),
        );
        let model = appointment_repository::insert(&tx, active)
            .await
            .map_err(|e| e.to_string())?;
        ids.push(model.id);
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for id in ids {
        results.push(appointment_repository::find_by_id(db, id).await?);
    }
    Ok(results)
}

/// Updates an appointment and returns the updated record.
pub async fn update(
    db: &DatabaseConnection,
    input: UpdateAppointmentInput,
) -> Result<Appointment, String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let active = appointments::ActiveModel {
        id: Set(input.id),
        client_id: Set(input.client_id),
        service_id: Set(input.service_id),
        date: Set(input.date),
        start_time: Set(input.start_time),
        end_time: Set(input.end_time),
        status: Set(input.status.as_str().to_owned()),
        notes: Set(Some(input.notes)),
        recurrence_group_id: Set(input.recurrence_group_id),
        updated_at: Set(now),
        ..Default::default()
    };

    appointment_repository::update(db, active)
        .await
        .map_err(|e| e.to_string())?;
    appointment_repository::find_by_id(db, input.id).await
}

/// Removes an appointment by id.
pub async fn remove(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    appointment_repository::delete(db, id)
        .await
        .map_err(|e| e.to_string())
}

// ─── Private helpers ──────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn build_active(
    client_id: i64,
    service_id: Option<i64>,
    date: &str,
    start_time: &str,
    end_time: &str,
    status: &str,
    notes: Option<String>,
    recurrence_group_id: Option<i64>,
) -> appointments::ActiveModel {
    appointments::ActiveModel {
        client_id: Set(client_id),
        service_id: Set(service_id),
        date: Set(date.to_owned()),
        start_time: Set(start_time.to_owned()),
        end_time: Set(end_time.to_owned()),
        status: Set(status.to_owned()),
        notes: Set(notes),
        recurrence_group_id: Set(recurrence_group_id),
        ..Default::default()
    }
}
