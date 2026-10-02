use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait};

use crate::app::common::AppError;
use crate::app::entity::email_template::{self, ActiveModel};

/// Returns the singleton template row, or None while the built-in one is in use.
pub async fn find(db: &impl ConnectionTrait) -> Result<Option<email_template::Model>, AppError> {
    email_template::Entity::find_by_id(1)
        .one(db)
        .await
        .map_err(AppError::from)
}

pub async fn save(db: &impl ConnectionTrait, active: ActiveModel) -> Result<(), AppError> {
    if find(db).await?.is_some() {
        active.update(db).await?;
    } else {
        email_template::Entity::insert(active).exec(db).await?;
    }
    Ok(())
}

/// Forgets the saved template, so the built-in one applies again.
pub async fn delete(db: &impl ConnectionTrait) -> Result<(), AppError> {
    email_template::Entity::delete_by_id(1).exec(db).await?;
    Ok(())
}
