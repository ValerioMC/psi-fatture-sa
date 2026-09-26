use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240401_add_specialization_and_quantity_option"
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

const UP_SQL: &str = "
ALTER TABLE professional_config ADD COLUMN specialization TEXT NOT NULL DEFAULT '';
ALTER TABLE professional_config ADD COLUMN hide_quantity_in_invoice INTEGER NOT NULL DEFAULT 0;
";
