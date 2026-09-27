use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240601_add_email"
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

/// The mailbox invoices leave from, the template they start from, and a row per
/// attempt to email an invoice. The mailbox password lives in the secrets file.
const UP_SQL: &str = "
CREATE TABLE IF NOT EXISTS email_account (
    id             INTEGER PRIMARY KEY CHECK (id = 1),
    provider       TEXT NOT NULL CHECK (provider IN ('psypec', 'aruba_pec', 'gmail', 'custom')),
    sender_address TEXT NOT NULL,
    sender_name    TEXT NOT NULL DEFAULT '',
    smtp_host      TEXT NOT NULL,
    smtp_port      INTEGER NOT NULL CHECK (smtp_port BETWEEN 1 AND 65535),
    security       TEXT NOT NULL CHECK (security IN ('tls', 'starttls')),
    username       TEXT NOT NULL,
    bcc_self       INTEGER NOT NULL DEFAULT 0,
    updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS email_template (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    subject    TEXT NOT NULL,
    body       TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS invoice_emails (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id      INTEGER NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    recipient       TEXT NOT NULL,
    subject         TEXT NOT NULL,
    attachment_name TEXT NOT NULL,
    status          TEXT NOT NULL CHECK (status IN ('sent', 'failed')),
    error           TEXT,
    sent_at         TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_invoice_emails_invoice ON invoice_emails(invoice_id, sent_at);
";
