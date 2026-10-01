use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::app::entity::terms_acceptance::{self, ActiveModel, Column};

pub async fn find_by_version(
    db: &impl ConnectionTrait,
    version: &str,
) -> Result<Option<terms_acceptance::Model>, String> {
    terms_acceptance::Entity::find()
        .filter(Column::Version.eq(version))
        .one(db)
        .await
        .map_err(|e| e.to_string())
}

pub async fn insert(
    db: &impl ConnectionTrait,
    active: ActiveModel,
) -> Result<terms_acceptance::Model, String> {
    active.insert(db).await.map_err(|e| e.to_string())
}
