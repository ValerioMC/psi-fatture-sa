use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "clients")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub client_type: String,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub fiscal_code: String,
    pub vat_number: Option<String>,
    pub address: String,
    pub city: String,
    pub province: String,
    pub zip_code: String,
    pub email: Option<String>,
    pub phone: String,
    pub notes: Option<String>,
    pub sts_authorization: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::invoice::Entity")]
    Invoices,
    #[sea_orm(has_many = "super::appointment::Entity")]
    Appointments,
}

impl Related<super::invoice::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Invoices.def()
    }
}

impl Related<super::appointment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Appointments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
