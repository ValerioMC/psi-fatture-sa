use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "invoices")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub client_id: i64,
    pub invoice_number: String,
    pub year: i64,
    pub issue_date: String,
    pub due_date: Option<String>,
    pub status: String,
    pub payment_method: String,
    pub notes: Option<String>,
    pub apply_enpap: i32,
    pub contributo_enpap: f64,
    pub ritenuta_acconto: f64,
    pub marca_da_bollo: i32,
    pub total_net: f64,
    pub total_tax: f64,
    pub total_gross: f64,
    pub total_due: f64,
    pub paid_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::client::Entity",
        from = "Column::ClientId",
        to = "super::client::Column::Id"
    )]
    Client,
    #[sea_orm(has_many = "super::invoice_line::Entity")]
    InvoiceLines,
}

impl Related<super::client::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Client.def()
    }
}

impl Related<super::invoice_line::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InvoiceLines.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
