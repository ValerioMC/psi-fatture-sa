use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240901_add_whatsapp"
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

/// The WhatsApp tables of a feature that was dropped; kept so databases that already
/// applied it stay consistent. `m20241001_drop_whatsapp` removes them.
const UP_SQL: &str = "
CREATE TABLE IF NOT EXISTS whatsapp_account (
    id                   INTEGER PRIMARY KEY CHECK (id = 1),
    phone_number_id      TEXT NOT NULL,
    template_name        TEXT NOT NULL,
    template_language    TEXT NOT NULL,
    default_country_code TEXT NOT NULL,
    updated_at           TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS invoice_whatsapps (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id      INTEGER NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    recipient       TEXT NOT NULL,
    attachment_name TEXT NOT NULL,
    status          TEXT NOT NULL CHECK (status IN ('sent', 'failed')),
    error           TEXT,
    sent_at         TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_invoice_whatsapps_invoice ON invoice_whatsapps(invoice_id, sent_at);
";
