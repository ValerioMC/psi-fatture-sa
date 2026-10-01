use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240701_add_invoice_amount_options"
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

/// Existing invoices keep the quantity visibility the profile gave them when they were printed.
/// A NULL on a client means it follows the profile.
const UP_SQL: &str = "
ALTER TABLE professional_config ADD COLUMN enpap_excludes_bollo INTEGER NOT NULL DEFAULT 0;
ALTER TABLE clients ADD COLUMN hide_quantity_in_invoice INTEGER;
ALTER TABLE invoices ADD COLUMN hide_quantity INTEGER NOT NULL DEFAULT 0;
UPDATE invoices SET hide_quantity =
    COALESCE((SELECT hide_quantity_in_invoice FROM professional_config WHERE id = 1), 0);
ALTER TABLE invoice_lines ADD COLUMN amount_override REAL;
";
