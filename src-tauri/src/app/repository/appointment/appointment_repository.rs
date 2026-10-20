use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait, FromQueryResult, Statement,
};

use super::AppointmentRow;
use crate::app::common::AppError;
use crate::app::entity::{appointment as appointments, recurrence_group as recurrence_groups};
use crate::app::model::appointment::Appointment;

/// Returns all appointments matching the given optional filters.
pub async fn find_all(
    db: &impl ConnectionTrait,
    date_from: Option<String>,
    date_to: Option<String>,
    client_id: Option<i64>,
) -> Result<Vec<Appointment>, AppError> {
    let (conditions, values) = filter_clauses(date_from, date_to, client_id);
    let sql = format!(
        "{APPOINTMENT_SELECT} WHERE {} ORDER BY a.date, a.start_time",
        conditions.join(" AND ")
    );
    load_appointments_by_sql(db, &sql, values).await
}

/// The appointment columns with the client and service names, up to the `WHERE`.
const APPOINTMENT_SELECT: &str =
    "SELECT a.id, a.client_id, a.service_id, a.date, a.start_time, a.end_time,
                a.status, a.notes, a.recurrence_group_id, a.invoice_id,
                a.created_at, a.updated_at,
                c.first_name || ' ' || c.last_name AS client_name,
                s.name AS service_name
         FROM appointments a
         JOIN clients c ON a.client_id = c.id
         LEFT JOIN services s ON a.service_id = s.id";

fn filter_clauses(
    date_from: Option<String>,
    date_to: Option<String>,
    client_id: Option<i64>,
) -> (Vec<&'static str>, Vec<sea_orm::Value>) {
    let mut conditions = vec!["1=1"];
    let mut values: Vec<sea_orm::Value> = Vec::new();
    if let Some(date) = date_from {
        conditions.push("a.date >= ?");
        values.push(date.into());
    }
    if let Some(date) = date_to {
        conditions.push("a.date <= ?");
        values.push(date.into());
    }
    if let Some(client) = client_id {
        conditions.push("a.client_id = ?");
        values.push(client.into());
    }
    (conditions, values)
}

/// The appointment, or `NotFound`: the one lookup that requires it to exist.
pub async fn require(db: &impl ConnectionTrait, id: i64) -> Result<Appointment, AppError> {
    let sql = format!("{APPOINTMENT_SELECT} WHERE a.id = ?");

    let mut results = load_appointments_by_sql(db, &sql, vec![id.into()]).await?;
    results
        .pop()
        .ok_or_else(|| AppError::NotFound(format!("Appuntamento {id} non trovato")))
}

/// Inserts a new appointment record.
pub async fn insert(
    db: &impl sea_orm::ConnectionTrait,
    active: appointments::ActiveModel,
) -> Result<appointments::Model, sea_orm::DbErr> {
    active.insert(db).await
}

/// Updates an appointment record.
pub async fn update(
    db: &impl ConnectionTrait,
    active: appointments::ActiveModel,
) -> Result<appointments::Model, sea_orm::DbErr> {
    active.update(db).await
}

/// Deletes an appointment by id.
pub async fn delete(db: &impl ConnectionTrait, id: i64) -> Result<(), sea_orm::DbErr> {
    appointments::Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}

/// Inserts a recurrence group for a client and returns its id.
pub async fn insert_recurrence_group(
    db: &impl sea_orm::ConnectionTrait,
    client_id: i64,
) -> Result<i64, sea_orm::DbErr> {
    let active = recurrence_groups::ActiveModel {
        client_id: Set(client_id),
        ..Default::default()
    };
    let model = active.insert(db).await?;
    Ok(model.id)
}

/// Returns completed appointments for the given month that have not yet been invoiced.
pub async fn find_unbilled_for_month(
    db: &impl ConnectionTrait,
    year: i64,
    month: i64,
) -> Result<Vec<Appointment>, AppError> {
    let date_from = format!("{year:04}-{month:02}-01");
    let date_to = if month == 12 {
        format!("{:04}-01-01", year + 1)
    } else {
        format!("{year:04}-{:02}-01", month + 1)
    };

    let sql = "SELECT a.id, a.client_id, a.service_id, a.date, a.start_time, a.end_time,
                a.status, a.notes, a.recurrence_group_id, a.invoice_id,
                a.created_at, a.updated_at,
                c.first_name || ' ' || c.last_name AS client_name,
                s.name AS service_name
         FROM appointments a
         JOIN clients c ON a.client_id = c.id
         LEFT JOIN services s ON a.service_id = s.id
         WHERE a.status = 'completed'
           AND a.invoice_id IS NULL
           AND a.date >= ?
           AND a.date < ?
         ORDER BY a.client_id, a.date, a.start_time";

    load_appointments_by_sql(db, sql, vec![date_from.into(), date_to.into()]).await
}

/// Links a set of appointment ids to an invoice.
pub async fn mark_as_invoiced(
    db: &impl sea_orm::ConnectionTrait,
    appointment_ids: &[i64],
    invoice_id: i64,
) -> Result<(), AppError> {
    if appointment_ids.is_empty() {
        return Ok(());
    }
    let placeholders = vec!["?"; appointment_ids.len()].join(", ");
    let sql = format!("UPDATE appointments SET invoice_id = ? WHERE id IN ({placeholders})");
    let mut values: Vec<sea_orm::Value> = vec![invoice_id.into()];
    for id in appointment_ids {
        values.push((*id).into());
    }
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        &sql,
        values,
    ))
    .await?;
    Ok(())
}

// ─── Private helpers ──────────────────────────────────────────────────────────

async fn load_appointments_by_sql(
    db: &impl ConnectionTrait,
    sql: &str,
    values: Vec<sea_orm::Value>,
) -> Result<Vec<Appointment>, AppError> {
    let rows = AppointmentRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        sql,
        values,
    ))
    .all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(AppointmentRow::into_appointment)
        .collect())
}
