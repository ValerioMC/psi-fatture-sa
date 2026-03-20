use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "appointments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub client_id: i64,
    pub service_id: Option<i64>,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
    pub status: String,
    pub notes: Option<String>,
    pub recurrence_group_id: Option<i64>,
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
    #[sea_orm(
        belongs_to = "super::service::Entity",
        from = "Column::ServiceId",
        to = "super::service::Column::Id"
    )]
    Service,
    #[sea_orm(
        belongs_to = "super::recurrence_group::Entity",
        from = "Column::RecurrenceGroupId",
        to = "super::recurrence_group::Column::Id"
    )]
    RecurrenceGroup,
}

impl Related<super::client::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Client.def()
    }
}

impl Related<super::service::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Service.def()
    }
}

impl Related<super::recurrence_group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RecurrenceGroup.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
