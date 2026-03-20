/// Tauri commands for appointment CRUD operations.
use crate::db;
use crate::domain::models::{
    Appointment, AppointmentStatus, CreateAppointmentInput, CreateRecurringAppointmentsInput,
    UpdateAppointmentInput,
};
use rusqlite::params;

#[tauri::command]
pub fn list_appointments(
    date_from: Option<String>,
    date_to: Option<String>,
    client_id: Option<i64>,
) -> Result<Vec<Appointment>, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let mut conditions = vec!["1=1".to_string()];
    if let Some(d) = &date_from {
        conditions.push(format!("a.date >= '{d}'"));
    }
    if let Some(d) = &date_to {
        conditions.push(format!("a.date <= '{d}'"));
    }
    if let Some(cid) = client_id {
        conditions.push(format!("a.client_id = {cid}"));
    }

    let where_clause = conditions.join(" AND ");
    let sql = format!(
        "SELECT a.*,
                c.first_name || ' ' || c.last_name as client_name,
                s.name as service_name
         FROM appointments a
         JOIN clients c ON a.client_id = c.id
         LEFT JOIN services s ON a.service_id = s.id
         WHERE {where_clause}
         ORDER BY a.date, a.start_time"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], row_to_appointment)
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn get_appointment(id: i64) -> Result<Appointment, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT a.*,
                c.first_name || ' ' || c.last_name as client_name,
                s.name as service_name
         FROM appointments a
         JOIN clients c ON a.client_id = c.id
         LEFT JOIN services s ON a.service_id = s.id
         WHERE a.id = ?1",
        params![id],
        row_to_appointment,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_appointment(input: CreateAppointmentInput) -> Result<Appointment, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO appointments
            (client_id, service_id, date, start_time, end_time, status, notes, recurrence_group_id)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            input.client_id, input.service_id, input.date, input.start_time,
            input.end_time, input.status.as_str(), input.notes, input.recurrence_group_id
        ],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    get_appointment(id)
}

#[tauri::command]
pub fn create_recurring_appointments(
    input: CreateRecurringAppointmentsInput,
) -> Result<Vec<Appointment>, String> {
    let mut conn = db::open().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO recurrence_groups (client_id) VALUES (?1)",
        params![input.client_id],
    )
    .map_err(|e| e.to_string())?;
    let group_id = tx.last_insert_rowid();

    let mut ids = Vec::new();
    for date in &input.dates {
        tx.execute(
            "INSERT INTO appointments
                (client_id, service_id, date, start_time, end_time, status, notes, recurrence_group_id)
             VALUES (?1,?2,?3,?4,?5,'scheduled',?6,?7)",
            params![
                input.client_id, input.service_id, date,
                input.start_time, input.end_time, input.notes, group_id
            ],
        )
        .map_err(|e| e.to_string())?;
        ids.push(tx.last_insert_rowid());
    }

    tx.commit().map_err(|e| e.to_string())?;

    ids.iter().map(|&id| get_appointment(id)).collect()
}

#[tauri::command]
pub fn update_appointment(input: UpdateAppointmentInput) -> Result<Appointment, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let rows = conn
        .execute(
            "UPDATE appointments SET
                client_id=?1, service_id=?2, date=?3, start_time=?4, end_time=?5,
                status=?6, notes=?7, recurrence_group_id=?8, updated_at=datetime('now')
             WHERE id=?9",
            params![
                input.client_id, input.service_id, input.date, input.start_time,
                input.end_time, input.status.as_str(), input.notes,
                input.recurrence_group_id, input.id
            ],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Appointment {} not found", input.id));
    }
    get_appointment(input.id)
}

#[tauri::command]
pub fn delete_appointment(id: i64) -> Result<(), String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM appointments WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn row_to_appointment(row: &rusqlite::Row) -> rusqlite::Result<Appointment> {
    Ok(Appointment {
        id: row.get(0)?,
        client_id: row.get(1)?,
        service_id: row.get(2)?,
        date: row.get(3)?,
        start_time: row.get(4)?,
        end_time: row.get(5)?,
        status: AppointmentStatus::from(row.get::<_, String>(6)?),
        notes: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
        recurrence_group_id: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        client_name: row.get(11)?,
        service_name: row.get(12)?,
    })
}
