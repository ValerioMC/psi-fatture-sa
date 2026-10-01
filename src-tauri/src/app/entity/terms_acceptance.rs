use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "terms_acceptances")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub version: String,
    pub clauses_approved: bool,
    pub accepted_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
