use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait};

use crate::app::entity::email_account::{self, ActiveModel};

/// Returns the singleton mailbox row, or None before the first save.
pub async fn find(db: &impl ConnectionTrait) -> Result<Option<email_account::Model>, String> {
    email_account::Entity::find_by_id(1)
        .one(db)
        .await
        .map_err(|e| e.to_string())
}

pub async fn save(db: &impl ConnectionTrait, active: ActiveModel) -> Result<(), String> {
    if find(db).await?.is_some() {
        active.update(db).await.map_err(|e| e.to_string())?;
    } else {
        email_account::Entity::insert(active)
            .exec(db)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
