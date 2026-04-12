use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, FromQueryResult, Statement,
};

use crate::app::entity::{appointment as appointments, recurrence_group as recurrence_groups};
use crate::app::model::appointment::{Appointment, AppointmentStatus};

/// Returns all appointments matching the given optional filters.
pub async fn find_all(
    db: &DatabaseConnection,
    date_from: Option<String>,
    date_to: Option<String>,
    client_id: Option<i64>,
) -> Result<Vec<Appointment>, String> {
    let mut conditions = vec!["1=1".to_string()];
    if let Some(d) = date_from {
        conditions.push(format!("a.date >= '{d}'"));
    }
    if let Some(d) = date_to {
        conditions.push(format!("a.date <= '{d}'"));
    }
    if let Some(c) = client_id {
        conditions.push(format!("a.client_id = {c}"));
    }

    let sql = format!(
        "SELECT a.id, a.client_id, a.service_id, a.date, a.start_time, a.end_time,
                a.status, a.notes, a.recurrence_group_id, a.invoice_id,
                a.created_at, a.updated_at,
                c.first_name || ' ' || c.last_name AS client_name,
                s.name AS service_name
         FROM appointments a
         JOIN clients c ON a.client_id = c.id
         LEFT JOIN services s ON a.service_id = s.id
         WHERE {} ORDER BY a.date, a.start_time",
        conditions.join(" AND ")
    );

    load_appointments_by_sql(db, sql).await
}

/// Returns a single appointment by id.
pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> Result<Appointment, String> {
    let sql = format!(
        "SELECT a.id, a.client_id, a.service_id, a.date, a.start_time, a.end_time,
                a.status, a.notes, a.recurrence_group_id, a.invoice_id,
                a.created_at, a.updated_at,
                c.first_name || ' ' || c.last_name AS client_name,
                s.name AS service_name
         FROM appointments a
         JOIN clients c ON a.client_id = c.id
         LEFT JOIN services s ON a.service_id = s.id
         WHERE a.id = {id}"
    );

    let mut results = load_appointments_by_sql(db, sql).await?;
    results
        .pop()
        .ok_or_else(|| format!("Appointment {id} not found"))
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
    db: &DatabaseConnection,
    active: appointments::ActiveModel,
) -> Result<appointments::Model, sea_orm::DbErr> {
    active.update(db).await
}

/// Deletes an appointment by id.
pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<(), sea_orm::DbErr> {
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
    db: &DatabaseConnection,
    year: i64,
    month: i64,
) -> Result<Vec<Appointment>, String> {
    let date_from = format!("{year:04}-{month:02}-01");
    let date_to = if month == 12 {
        format!("{:04}-01-01", year + 1)
    } else {
        format!("{year:04}-{:02}-01", month + 1)
    };

    let sql = format!(
        "SELECT a.id, a.client_id, a.service_id, a.date, a.start_time, a.end_time,
                a.status, a.notes, a.recurrence_group_id, a.invoice_id,
                a.created_at, a.updated_at,
                c.first_name || ' ' || c.last_name AS client_name,
                s.name AS service_name
         FROM appointments a
         JOIN clients c ON a.client_id = c.id
         LEFT JOIN services s ON a.service_id = s.id
         WHERE a.status = 'completed'
           AND a.invoice_id IS NULL
           AND a.date >= '{date_from}'
           AND a.date < '{date_to}'
         ORDER BY a.client_id, a.date, a.start_time"
    );

    load_appointments_by_sql(db, sql).await
}

/// Links a set of appointment ids to an invoice.
pub async fn mark_as_invoiced(
    db: &impl sea_orm::ConnectionTrait,
    appointment_ids: &[i64],
    invoice_id: i64,
) -> Result<(), String> {
    if appointment_ids.is_empty() {
        return Ok(());
    }
    let ids_csv: String = appointment_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    db.execute_unprepared(&format!(
        "UPDATE appointments SET invoice_id = {invoice_id} WHERE id IN ({ids_csv})"
    ))
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Private helpers ──────────────────────────────────────────────────────────

async fn load_appointments_by_sql(
    db: &DatabaseConnection,
    sql: String,
) -> Result<Vec<Appointment>, String> {
    #[derive(FromQueryResult)]
    struct Row {
        id: i64,
        client_id: i64,
        service_id: Option<i64>,
        date: String,
        start_time: String,
        end_time: String,
        status: String,
        notes: Option<String>,
        recurrence_group_id: Option<i64>,
        invoice_id: Option<i64>,
        created_at: String,
        updated_at: String,
        client_name: String,
        service_name: Option<String>,
    }

    let rows = Row::find_by_statement(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        sql,
    ))
    .all(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| Appointment {
            id: r.id,
            client_id: r.client_id,
            client_name: r.client_name,
            service_id: r.service_id,
            service_name: r.service_name,
            date: r.date,
            start_time: r.start_time,
            end_time: r.end_time,
            status: AppointmentStatus::from(r.status),
            notes: r.notes.unwrap_or_default(),
            recurrence_group_id: r.recurrence_group_id,
            invoice_id: r.invoice_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}
