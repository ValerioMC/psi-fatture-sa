use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240201_add_batch_invoicing"
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
ALTER TABLE professional_config ADD COLUMN initial_invoice_number INTEGER NOT NULL DEFAULT 1;
ALTER TABLE appointments ADD COLUMN invoice_id INTEGER REFERENCES invoices(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_appointments_invoice ON appointments(invoice_id);
";
