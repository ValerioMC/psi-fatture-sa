use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait};

use crate::app::common::AppError;
use crate::app::entity::email_account::{self, ActiveModel};

/// Returns the singleton mailbox row, or None before the first save.
pub async fn find(db: &impl ConnectionTrait) -> Result<Option<email_account::Model>, AppError> {
    email_account::Entity::find_by_id(1)
        .one(db)
        .await
        .map_err(AppError::from)
}

pub async fn save(db: &impl ConnectionTrait, active: ActiveModel) -> Result<(), AppError> {
    if find(db).await?.is_some() {
        active.update(db).await?;
    } else {
        email_account::Entity::insert(active).exec(db).await?;
    }
    Ok(())
}
