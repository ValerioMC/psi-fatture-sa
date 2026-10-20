use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder,
};

use crate::app::entity::service::{self, ActiveModel, Column};

/// Returns all services ordered by name, optionally filtered to active-only.
pub async fn find_all(
    db: &impl ConnectionTrait,
    active_only: bool,
) -> Result<Vec<service::Model>, sea_orm::DbErr> {
    let mut query = service::Entity::find().order_by_asc(Column::Name);

    if active_only {
        query = query.filter(Column::IsActive.eq(1));
    }

    query.all(db).await
}

/// Inserts a new service and returns the created record.
pub async fn insert(
    db: &impl ConnectionTrait,
    active: ActiveModel,
) -> Result<service::Model, sea_orm::DbErr> {
    active.insert(db).await
}

/// Updates an existing service and returns the updated record.
pub async fn update(
    db: &impl ConnectionTrait,
    active: ActiveModel,
) -> Result<service::Model, sea_orm::DbErr> {
    active.update(db).await
}

/// Deletes a service by id.
pub async fn delete(db: &impl ConnectionTrait, id: i64) -> Result<(), sea_orm::DbErr> {
    service::Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}
