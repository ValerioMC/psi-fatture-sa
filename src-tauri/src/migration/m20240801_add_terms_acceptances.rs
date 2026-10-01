use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240801_add_terms_acceptances"
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

/// One row per accepted version of the terms of use, kept as the record of the specific approval.
const UP_SQL: &str = "
CREATE TABLE IF NOT EXISTS terms_acceptances (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    version          TEXT NOT NULL UNIQUE,
    clauses_approved INTEGER NOT NULL CHECK (clauses_approved = 1),
    accepted_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
";
