use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "professional_config")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub title: String,
    pub first_name: String,
    pub last_name: String,
    pub vat_number: String,
    pub fiscal_code: String,
    pub tax_regime: String,
    pub albo_number: String,
    pub albo_region: String,
    pub address: String,
    pub city: String,
    pub province: String,
    pub zip_code: String,
    pub country: String,
    pub phone: String,
    pub pec_email: String,
    pub iban: String,
    pub coefficient: f64,
    pub is_psicoanalista: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
