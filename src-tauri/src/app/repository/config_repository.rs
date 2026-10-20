use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait};

use crate::app::common::AppError;
use crate::app::entity::professional_config::{self, ActiveModel};

/// Returns the professional config (singleton, id=1), or None if not configured.
pub async fn find(
    db: &impl ConnectionTrait,
) -> Result<Option<professional_config::Model>, sea_orm::DbErr> {
    professional_config::Entity::find_by_id(1).one(db).await
}

/// The professional config, or `NotFound` while the profile has never been saved.
pub async fn require(db: &impl ConnectionTrait) -> Result<professional_config::Model, AppError> {
    find(db).await?.ok_or_else(|| {
        AppError::NotFound("Profilo professionale non ancora configurato".to_string())
    })
}

/// Inserts the singleton row on first setup, updates it afterwards.
pub async fn save(
    db: &impl ConnectionTrait,
    active: ActiveModel,
) -> Result<professional_config::Model, AppError> {
    if find(db).await?.is_some() {
        active.update(db).await?;
    } else {
        professional_config::Entity::insert(active).exec(db).await?;
    }

    require(db).await
}
