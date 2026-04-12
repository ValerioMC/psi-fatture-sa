use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};

use crate::app::entity::client::{self, ActiveModel, Column};
use crate::app::entity::invoice::{Column as InvCol, Entity as InvoiceEntity};

/// Returns all clients ordered by last name then first name, with optional search filter.
pub async fn find_all(
    db: &DatabaseConnection,
    search: Option<String>,
) -> Result<Vec<client::Model>, sea_orm::DbErr> {
    let mut query = client::Entity::find()
        .order_by_asc(Column::LastName)
        .order_by_asc(Column::FirstName);

    if let Some(q) = search.filter(|s| !s.trim().is_empty()) {
        let pattern = format!("%{}%", q.trim().to_lowercase());
        query = query.filter(
            Column::LastName
                .contains(&pattern)
                .or(Column::FirstName.contains(&pattern))
                .or(Column::FiscalCode.contains(&pattern))
                .or(Column::Email.contains(&pattern)),
        );
    }

    query.all(db).await
}

/// Returns a single client by id.
pub async fn find_by_id(
    db: &DatabaseConnection,
    id: i64,
) -> Result<Option<client::Model>, sea_orm::DbErr> {
    client::Entity::find_by_id(id).one(db).await
}

/// Inserts a new client and returns the created record.
pub async fn insert(
    db: &DatabaseConnection,
    active: ActiveModel,
) -> Result<client::Model, sea_orm::DbErr> {
    active.insert(db).await
}

/// Updates an existing client and returns the updated record.
pub async fn update(
    db: &DatabaseConnection,
    active: ActiveModel,
) -> Result<client::Model, sea_orm::DbErr> {
    active.update(db).await
}

/// Deletes a client by id.
pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<(), sea_orm::DbErr> {
    client::Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}

/// Returns true if the client has any associated invoices.
pub async fn has_invoices(db: &DatabaseConnection, client_id: i64) -> Result<bool, sea_orm::DbErr> {
    let found = InvoiceEntity::find()
        .filter(InvCol::ClientId.eq(client_id))
        .one(db)
        .await?;
    Ok(found.is_some())
}
