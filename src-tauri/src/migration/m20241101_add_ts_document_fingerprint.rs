use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20241101_add_ts_document_fingerprint"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(UP_SQL).await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

/// The fingerprint of the expense data a submission sent, so a later edit of
/// the invoice shows up as "the Sistema TS holds an older version".
const UP_SQL: &str = "
ALTER TABLE ts_submissions ADD COLUMN document_fingerprint TEXT;
";
