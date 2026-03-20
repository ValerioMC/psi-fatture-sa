/// Tauri commands for service CRUD operations.
use crate::db;
use crate::domain::models::{CreateServiceInput, Service, UpdateServiceInput};
use rusqlite::params;

#[tauri::command]
pub fn list_services(active_only: bool) -> Result<Vec<Service>, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let sql = if active_only {
        "SELECT * FROM services WHERE is_active = 1 ORDER BY name"
    } else {
        "SELECT * FROM services ORDER BY name"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], row_to_service)
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn get_service(id: i64) -> Result<Service, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT * FROM services WHERE id = ?1",
        params![id],
        row_to_service,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_service(input: CreateServiceInput) -> Result<Service, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO services (name, description, default_price, vat_rate, is_active)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            input.name, input.description, input.default_price,
            input.vat_rate, input.is_active as i64
        ],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    get_service(id)
}

#[tauri::command]
pub fn update_service(input: UpdateServiceInput) -> Result<Service, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let rows = conn
        .execute(
            "UPDATE services SET name=?1, description=?2, default_price=?3,
             vat_rate=?4, is_active=?5, updated_at=datetime('now')
             WHERE id=?6",
            params![
                input.name, input.description, input.default_price,
                input.vat_rate, input.is_active as i64, input.id
            ],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Service {} not found", input.id));
    }
    get_service(input.id)
}

#[tauri::command]
pub fn delete_service(id: i64) -> Result<(), String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM services WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn row_to_service(row: &rusqlite::Row) -> rusqlite::Result<Service> {
    Ok(Service {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        default_price: row.get(3)?,
        vat_rate: row.get(4)?,
        is_active: row.get::<_, i64>(5)? != 0,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}
