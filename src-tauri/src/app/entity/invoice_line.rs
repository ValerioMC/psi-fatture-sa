use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "invoice_lines")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub invoice_id: i64,
    pub service_id: Option<i64>,
    pub description: String,
    pub quantity: i64,
    pub unit_price: f64,
    pub vat_rate: f64,
    pub line_total: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::invoice::Entity",
        from = "Column::InvoiceId",
        to = "super::invoice::Column::Id"
    )]
    Invoice,
}

impl Related<super::invoice::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Invoice.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
