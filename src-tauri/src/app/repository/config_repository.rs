use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};

use crate::app::entity::professional_config::{self, ActiveModel};

/// Returns the professional config (singleton, id=1), or None if not configured.
pub async fn find(
    db: &DatabaseConnection,
) -> Result<Option<professional_config::Model>, sea_orm::DbErr> {
    professional_config::Entity::find_by_id(1).one(db).await
}

/// Saves (upsert) the professional config and returns the updated record.
///
/// Uses INSERT OR REPLACE so it works both on first setup and on subsequent updates.
pub async fn save(
    db: &DatabaseConnection,
    active: ActiveModel,
) -> Result<professional_config::Model, sea_orm::DbErr> {
    let existing = professional_config::Entity::find_by_id(1).one(db).await?;

    if existing.is_some() {
        active.update(db).await?;
    } else {
        professional_config::Entity::insert(active).exec(db).await?;
    }

    professional_config::Entity::find_by_id(1)
        .one(db)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Config not found after save".to_string()))
}
