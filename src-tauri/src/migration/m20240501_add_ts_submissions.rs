use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240501_add_ts_submissions"
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

/// Sistema TS settings and the offline transmission queue. `target_submission_id`
/// links a cancellation or replacement to the accepted submission it acts on; the
/// `document_*` columns keep the id the document was sent under.
const UP_SQL: &str = "
CREATE TABLE IF NOT EXISTS ts_settings (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    environment TEXT NOT NULL DEFAULT 'produzione' CHECK (environment IN ('test', 'produzione')),
    username    TEXT NOT NULL DEFAULT '',
    vat_number  TEXT NOT NULL DEFAULT '',
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS ts_submissions (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id           INTEGER NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    operation            TEXT NOT NULL
                         CHECK (operation IN ('invio', 'sostituzione', 'annullamento')),
    status               TEXT NOT NULL DEFAULT 'non_inviata'
                         CHECK (status IN ('non_inviata', 'inviata', 'accettata',
                                           'scartata', 'annullata', 'sostituita')),
    target_submission_id INTEGER REFERENCES ts_submissions(id),
    environment          TEXT NOT NULL CHECK (environment IN ('test', 'produzione')),
    document_vat_number  TEXT,
    document_issue_date  TEXT,
    document_number      TEXT,
    protocol             TEXT,
    outcome_code         TEXT,
    outcome_message      TEXT,
    attempt_count        INTEGER NOT NULL DEFAULT 0,
    last_error           TEXT,
    next_attempt_at      TEXT NOT NULL DEFAULT (datetime('now')),
    last_attempt_at      TEXT,
    sent_at              TEXT,
    resolved_at          TEXT,
    created_at           TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at           TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK ((operation = 'invio') = (target_submission_id IS NULL))
);
CREATE INDEX IF NOT EXISTS idx_ts_submissions_invoice ON ts_submissions(invoice_id);
CREATE INDEX IF NOT EXISTS idx_ts_submissions_due ON ts_submissions(status, next_attempt_at);
CREATE UNIQUE INDEX IF NOT EXISTS idx_ts_submissions_in_flight
    ON ts_submissions(invoice_id) WHERE status IN ('non_inviata', 'inviata');
";
