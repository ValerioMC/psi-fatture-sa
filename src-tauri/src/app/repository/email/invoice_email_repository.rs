use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder,
};

use crate::app::entity::invoice_email::{self, ActiveModel, Column};
use crate::app::model::email::InvoiceEmailFilters;

pub async fn insert(
    db: &impl ConnectionTrait,
    active: ActiveModel,
) -> Result<invoice_email::Model, String> {
    active.insert(db).await.map_err(|e| e.to_string())
}

/// Attempts matching the filters, newest first.
pub async fn find(
    db: &impl ConnectionTrait,
    filters: &InvoiceEmailFilters,
) -> Result<Vec<invoice_email::Model>, String> {
    let mut query = invoice_email::Entity::find();
    if let Some(invoice_id) = filters.invoice_id {
        query = query.filter(Column::InvoiceId.eq(invoice_id));
    }
    query
        .order_by_desc(Column::SentAt)
        .order_by_desc(Column::Id)
        .all(db)
        .await
        .map_err(|e| e.to_string())
}
