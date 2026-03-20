use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "recurrence_groups")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub client_id: i64,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::client::Entity",
        from = "Column::ClientId",
        to = "super::client::Column::Id"
    )]
    Client,
    #[sea_orm(has_many = "super::appointment::Entity")]
    Appointments,
}

impl Related<super::client::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Client.def()
    }
}

impl Related<super::appointment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Appointments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
