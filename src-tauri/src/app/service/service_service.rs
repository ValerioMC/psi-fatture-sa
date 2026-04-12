use sea_orm::{ActiveValue::Set, DatabaseConnection};

use crate::app::entity::service::{self, ActiveModel};
use crate::app::model::service::{CreateServiceInput, Service, UpdateServiceInput};
use crate::app::repository::service_repository;

/// Lists all services, optionally filtered to active-only.
pub async fn list(db: &DatabaseConnection, active_only: bool) -> Result<Vec<Service>, String> {
    let models = service_repository::find_all(db, active_only)
        .await
        .map_err(|e| e.to_string())?;
    Ok(models.into_iter().map(into_domain).collect())
}

/// Returns a single service by id.
pub async fn get(db: &DatabaseConnection, id: i64) -> Result<Service, String> {
    service_repository::find_by_id(db, id)
        .await
        .map_err(|e| e.to_string())?
        .map(into_domain)
        .ok_or_else(|| format!("Service {id} not found"))
}

/// Creates a new service and returns the created record.
pub async fn create(db: &DatabaseConnection, input: CreateServiceInput) -> Result<Service, String> {
    let active = ActiveModel {
        name: Set(input.name),
        description: Set(input.description),
        default_price: Set(input.default_price),
        vat_rate: Set(input.vat_rate),
        is_active: Set(input.is_active as i32),
        ..Default::default()
    };

    let model = service_repository::insert(db, active)
        .await
        .map_err(|e| e.to_string())?;
    Ok(into_domain(model))
}

/// Updates an existing service and returns the updated record.
pub async fn update(db: &DatabaseConnection, input: UpdateServiceInput) -> Result<Service, String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let active = ActiveModel {
        id: Set(input.id),
        name: Set(input.name),
        description: Set(input.description),
        default_price: Set(input.default_price),
        vat_rate: Set(input.vat_rate),
        is_active: Set(input.is_active as i32),
        updated_at: Set(now),
        ..Default::default()
    };

    let model = service_repository::update(db, active)
        .await
        .map_err(|e| e.to_string())?;
    Ok(into_domain(model))
}

/// Removes a service by id.
pub async fn remove(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    service_repository::delete(db, id)
        .await
        .map_err(|e| e.to_string())
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn into_domain(m: service::Model) -> Service {
    Service {
        id: m.id,
        name: m.name,
        description: m.description,
        default_price: m.default_price,
        vat_rate: m.vat_rate,
        is_active: m.is_active != 0,
        created_at: m.created_at,
        updated_at: m.updated_at,
    }
}
