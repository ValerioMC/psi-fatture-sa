use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait};

use crate::app::common::AppError;
use crate::app::entity::ts_setting::{self, ActiveModel};

/// Returns the singleton Sistema TS settings row, or None before the first save.
pub async fn find(db: &impl ConnectionTrait) -> Result<Option<ts_setting::Model>, AppError> {
    ts_setting::Entity::find_by_id(1)
        .one(db)
        .await
        .map_err(AppError::from)
}

pub async fn save(db: &impl ConnectionTrait, active: ActiveModel) -> Result<(), AppError> {
    if find(db).await?.is_some() {
        active.update(db).await?;
    } else {
        ts_setting::Entity::insert(active).exec(db).await?;
    }
    Ok(())
}
