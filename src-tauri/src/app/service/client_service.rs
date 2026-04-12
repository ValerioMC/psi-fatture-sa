use sea_orm::{ActiveValue::Set, DatabaseConnection};

use crate::app::entity::client::{self, ActiveModel};
use crate::app::model::client::{Client, ClientType, CreateClientInput, UpdateClientInput};
use crate::app::repository::client_repository;

/// Lists all clients, optionally filtered by search query.
pub async fn list(db: &DatabaseConnection, search: Option<String>) -> Result<Vec<Client>, String> {
    let models = client_repository::find_all(db, search)
        .await
        .map_err(|e| e.to_string())?;
    Ok(models.into_iter().map(into_domain).collect())
}

/// Returns a single client by id.
pub async fn get(db: &DatabaseConnection, id: i64) -> Result<Client, String> {
    client_repository::find_by_id(db, id)
        .await
        .map_err(|e| e.to_string())?
        .map(into_domain)
        .ok_or_else(|| format!("Client {id} not found"))
}

/// Creates a new client and returns the created record.
pub async fn create(db: &DatabaseConnection, input: CreateClientInput) -> Result<Client, String> {
    let active = ActiveModel {
        client_type: Set(input.client_type.as_str().to_owned()),
        first_name: Set(input.first_name),
        last_name: Set(input.last_name),
        birth_date: Set(input.birth_date),
        gender: Set(input.gender),
        fiscal_code: Set(input.fiscal_code),
        vat_number: Set(input.vat_number),
        address: Set(input.address),
        city: Set(input.city),
        province: Set(input.province),
        zip_code: Set(input.zip_code),
        email: Set(input.email),
        phone: Set(input.phone),
        notes: Set(input.notes),
        sts_authorization: Set(input.sts_authorization as i32),
        ..Default::default()
    };

    let model = client_repository::insert(db, active)
        .await
        .map_err(|e| e.to_string())?;
    Ok(into_domain(model))
}

/// Updates an existing client and returns the updated record.
pub async fn update(db: &DatabaseConnection, input: UpdateClientInput) -> Result<Client, String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let active = ActiveModel {
        id: Set(input.id),
        client_type: Set(input.client_type.as_str().to_owned()),
        first_name: Set(input.first_name),
        last_name: Set(input.last_name),
        birth_date: Set(input.birth_date),
        gender: Set(input.gender),
        fiscal_code: Set(input.fiscal_code),
        vat_number: Set(input.vat_number),
        address: Set(input.address),
        city: Set(input.city),
        province: Set(input.province),
        zip_code: Set(input.zip_code),
        email: Set(input.email),
        phone: Set(input.phone),
        notes: Set(input.notes),
        sts_authorization: Set(input.sts_authorization as i32),
        updated_at: Set(now),
        ..Default::default()
    };

    let model = client_repository::update(db, active)
        .await
        .map_err(|e| e.to_string())?;
    Ok(into_domain(model))
}

/// Removes a client. Fails if the client has associated invoices.
pub async fn remove(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    let has = client_repository::has_invoices(db, id)
        .await
        .map_err(|e| e.to_string())?;
    if has {
        return Err("Impossibile eliminare: il cliente ha fatture associate".to_string());
    }
    client_repository::delete(db, id)
        .await
        .map_err(|e| e.to_string())
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn into_domain(m: client::Model) -> Client {
    Client {
        id: m.id,
        client_type: ClientType::from(m.client_type),
        first_name: m.first_name,
        last_name: m.last_name,
        birth_date: m.birth_date,
        gender: m.gender,
        fiscal_code: m.fiscal_code,
        vat_number: m.vat_number,
        address: m.address,
        city: m.city,
        province: m.province,
        zip_code: m.zip_code,
        email: m.email,
        phone: m.phone,
        notes: m.notes,
        sts_authorization: m.sts_authorization != 0,
        created_at: m.created_at,
        updated_at: m.updated_at,
    }
}
