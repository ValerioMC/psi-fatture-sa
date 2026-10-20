use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait};

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

/// Inserts the singleton row on first setup, updates it afterwards. `now` stamps
/// `updated_at` on every save and `created_at` only on the insert.
pub async fn save(
    db: &impl ConnectionTrait,
    mut active: ActiveModel,
    now: &str,
) -> Result<professional_config::Model, AppError> {
    active.updated_at = Set(now.to_owned());
    if find(db).await?.is_some() {
        active.update(db).await?;
    } else {
        active.created_at = Set(now.to_owned());
        professional_config::Entity::insert(active).exec(db).await?;
    }

    require(db).await
}
