use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};

use crate::app::entity::service::{self, ActiveModel, Column};

/// Returns all services ordered by name, optionally filtered to active-only.
pub async fn find_all(
    db: &DatabaseConnection,
    active_only: bool,
) -> Result<Vec<service::Model>, sea_orm::DbErr> {
    let mut query = service::Entity::find().order_by_asc(Column::Name);

    if active_only {
        query = query.filter(Column::IsActive.eq(1));
    }

    query.all(db).await
}

/// Returns a single service by id.
pub async fn find_by_id(
    db: &DatabaseConnection,
    id: i64,
) -> Result<Option<service::Model>, sea_orm::DbErr> {
    service::Entity::find_by_id(id).one(db).await
}

/// Inserts a new service and returns the created record.
pub async fn insert(
    db: &DatabaseConnection,
    active: ActiveModel,
) -> Result<service::Model, sea_orm::DbErr> {
    active.insert(db).await
}

/// Updates an existing service and returns the updated record.
pub async fn update(
    db: &DatabaseConnection,
    active: ActiveModel,
) -> Result<service::Model, sea_orm::DbErr> {
    active.update(db).await
}

/// Deletes a service by id.
pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<(), sea_orm::DbErr> {
    service::Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}
