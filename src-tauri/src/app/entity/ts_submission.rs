use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ts_submissions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub invoice_id: i64,
    pub operation: String,
    pub status: String,
    pub target_submission_id: Option<i64>,
    pub environment: String,
    pub document_vat_number: Option<String>,
    pub document_issue_date: Option<String>,
    pub document_number: Option<String>,
    pub protocol: Option<String>,
    pub outcome_code: Option<String>,
    pub outcome_message: Option<String>,
    pub attempt_count: i64,
    pub last_error: Option<String>,
    pub next_attempt_at: String,
    pub last_attempt_at: Option<String>,
    pub sent_at: Option<String>,
    pub resolved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
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
