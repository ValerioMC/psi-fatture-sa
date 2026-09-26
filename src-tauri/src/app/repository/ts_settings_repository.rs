use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait};

use crate::app::entity::ts_setting::{self, ActiveModel};

/// Returns the singleton Sistema TS settings row, or None before the first save.
pub async fn find(db: &impl ConnectionTrait) -> Result<Option<ts_setting::Model>, String> {
    ts_setting::Entity::find_by_id(1)
        .one(db)
        .await
        .map_err(|e| e.to_string())
}

pub async fn save(db: &impl ConnectionTrait, active: ActiveModel) -> Result<(), String> {
    if find(db).await?.is_some() {
        active.update(db).await.map_err(|e| e.to_string())?;
    } else {
        ts_setting::Entity::insert(active)
            .exec(db)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
