/// Tauri commands for client CRUD operations.
use crate::db;
use crate::domain::models::{Client, ClientType, CreateClientInput, UpdateClientInput};
use rusqlite::params;

#[tauri::command]
pub fn list_clients(search: Option<String>) -> Result<Vec<Client>, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let (sql, client_search) = match &search {
        Some(q) if !q.trim().is_empty() => {
            let pattern = format!("%{}%", q.trim().to_lowercase());
            (
                "SELECT * FROM clients WHERE
                    lower(first_name) LIKE ?1 OR lower(last_name) LIKE ?1 OR
                    lower(fiscal_code) LIKE ?1 OR lower(email) LIKE ?1
                 ORDER BY last_name, first_name"
                    .to_string(),
                Some(pattern),
            )
        }
        _ => (
            "SELECT * FROM clients ORDER BY last_name, first_name".to_string(),
            None,
        ),
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let rows = match client_search {
        Some(p) => stmt.query_map(params![p], row_to_client),
        None => stmt.query_map([], row_to_client),
    }
    .map_err(|e| e.to_string())?;

    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn get_client(id: i64) -> Result<Client, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT * FROM clients WHERE id = ?1",
        params![id],
        row_to_client,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_client(input: CreateClientInput) -> Result<Client, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO clients
            (client_type, first_name, last_name, birth_date, gender, fiscal_code,
             vat_number, address, city, province, zip_code, email, phone, notes,
             sts_authorization)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
        params![
            input.client_type.as_str(), input.first_name, input.last_name,
            input.birth_date, input.gender, input.fiscal_code,
            input.vat_number, input.address, input.city, input.province,
            input.zip_code, input.email, input.phone, input.notes,
            input.sts_authorization as i64
        ],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    get_client(id)
}

#[tauri::command]
pub fn update_client(input: UpdateClientInput) -> Result<Client, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let rows = conn.execute(
        "UPDATE clients SET
            client_type=?1, first_name=?2, last_name=?3, birth_date=?4, gender=?5,
            fiscal_code=?6, vat_number=?7, address=?8, city=?9, province=?10,
            zip_code=?11, email=?12, phone=?13, notes=?14, sts_authorization=?15,
            updated_at=datetime('now')
         WHERE id=?16",
        params![
            input.client_type.as_str(), input.first_name, input.last_name,
            input.birth_date, input.gender, input.fiscal_code,
            input.vat_number, input.address, input.city, input.province,
            input.zip_code, input.email, input.phone, input.notes,
            input.sts_authorization as i64, input.id
        ],
    )
    .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Client {} not found", input.id));
    }
    get_client(input.id)
}

#[tauri::command]
pub fn delete_client(id: i64) -> Result<(), String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let has_invoices: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM invoices WHERE client_id = ?1)",
            params![id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    if has_invoices {
        return Err("Impossibile eliminare: il cliente ha fatture associate".to_string());
    }

    conn.execute("DELETE FROM clients WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn row_to_client(row: &rusqlite::Row) -> rusqlite::Result<Client> {
    Ok(Client {
        id: row.get(0)?,
        client_type: ClientType::from(row.get::<_, String>(1)?),
        first_name: row.get(2)?,
        last_name: row.get(3)?,
        birth_date: row.get(4)?,
        gender: row.get(5)?,
        fiscal_code: row.get(6)?,
        vat_number: row.get(7)?,
        address: row.get(8)?,
        city: row.get(9)?,
        province: row.get(10)?,
        zip_code: row.get(11)?,
        email: row.get(12)?,
        phone: row.get(13)?,
        notes: row.get(14)?,
        sts_authorization: row.get::<_, i64>(15)? != 0,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}
