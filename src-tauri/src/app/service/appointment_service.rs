use sea_orm::{ActiveValue::Set, ConnectionTrait, DatabaseConnection, TransactionTrait};

use crate::app::common::{AppError, Clock};
use crate::app::entity::appointment as appointments;
use crate::app::model::appointment::{
    Appointment, CreateAppointmentInput, CreateRecurringAppointmentsInput, UpdateAppointmentInput,
};
use crate::app::repository::appointment::appointment_repository;
use crate::app::service::support::validation as validate;

/// Lists appointments with optional date range and client filters.
pub async fn list(
    db: &DatabaseConnection,
    date_from: Option<String>,
    date_to: Option<String>,
    client_id: Option<i64>,
) -> Result<Vec<Appointment>, AppError> {
    appointment_repository::find_all(db, date_from, date_to, client_id).await
}

/// Creates a single appointment and returns it.
pub async fn create(
    db: &DatabaseConnection,
    clock: &dyn Clock,
    input: CreateAppointmentInput,
) -> Result<Appointment, AppError> {
    validate_appointment_fields(
        input.client_id,
        &input.date,
        &input.start_time,
        &input.end_time,
    )?;

    let active = build_active(
        input.client_id,
        input.service_id,
        &input.date,
        &input.start_time,
        &input.end_time,
        input.status.as_str(),
        Some(input.notes),
        input.recurrence_group_id,
        &clock.timestamp(),
    );

    let model = appointment_repository::insert(db, active).await?;
    appointment_repository::require(db, model.id).await
}

/// Creates multiple recurring appointments in a transaction.
pub async fn create_recurring(
    db: &DatabaseConnection,
    clock: &dyn Clock,
    input: CreateRecurringAppointmentsInput,
) -> Result<Vec<Appointment>, AppError> {
    validate_recurrence(&input)?;
    let tx = db.begin().await?;
    let ids = insert_recurrence(&tx, &input, &clock.timestamp()).await?;
    tx.commit().await?;

    let mut created = Vec::with_capacity(ids.len());
    for id in ids {
        created.push(appointment_repository::require(db, id).await?);
    }
    Ok(created)
}

/// Updates an appointment and returns the updated record.
pub async fn update(
    db: &DatabaseConnection,
    clock: &dyn Clock,
    input: UpdateAppointmentInput,
) -> Result<Appointment, AppError> {
    validate::validate_id(input.id, "Appuntamento")?;
    validate_appointment_fields(
        input.client_id,
        &input.date,
        &input.start_time,
        &input.end_time,
    )?;

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
        updated_at: Set(clock.timestamp()),
        ..Default::default()
    };

    appointment_repository::update(db, active).await?;
    appointment_repository::require(db, input.id).await
}

/// Removes an appointment by id.
pub async fn remove(db: &DatabaseConnection, id: i64) -> Result<(), AppError> {
    appointment_repository::delete(db, id)
        .await
        .map_err(AppError::from)
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn validate_recurrence(input: &CreateRecurringAppointmentsInput) -> Result<(), AppError> {
    if input.dates.is_empty() {
        return Err(AppError::Invalid(
            "Seleziona almeno una data per la ricorrenza".to_string(),
        ));
    }
    for date in &input.dates {
        validate_appointment_fields(input.client_id, date, &input.start_time, &input.end_time)?;
    }
    Ok(())
}

/// One recurrence group and a scheduled appointment per date; returns their ids in date order.
async fn insert_recurrence(
    db: &impl ConnectionTrait,
    input: &CreateRecurringAppointmentsInput,
    now: &str,
) -> Result<Vec<i64>, AppError> {
    let group_id =
        appointment_repository::insert_recurrence_group(db, input.client_id, now).await?;
    let mut ids = Vec::with_capacity(input.dates.len());
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
            now,
        );
        ids.push(appointment_repository::insert(db, active).await?.id);
    }
    Ok(ids)
}

/// Validates the shared fields of appointment inputs.
fn validate_appointment_fields(
    client_id: i64,
    date: &str,
    start_time: &str,
    end_time: &str,
) -> Result<(), AppError> {
    validate::validate_id(client_id, "Cliente")?;
    validate::parse_iso_date(date, "Data")?;
    validate::validate_time(start_time, "Ora inizio")?;
    validate::validate_time(end_time, "Ora fine")?;
    if end_time <= start_time {
        return Err(AppError::Invalid(
            "L'ora di fine deve essere successiva all'ora di inizio".to_string(),
        ));
    }
    Ok(())
}

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
    now: &str,
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
        created_at: Set(now.to_owned()),
        updated_at: Set(now.to_owned()),
        ..Default::default()
    }
}

#[cfg(test)]
#[path = "appointment_service_test.rs"]
mod tests;
