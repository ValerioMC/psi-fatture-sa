use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_create_schema"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(SCHEMA_SQL).await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS professional_config (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    title       TEXT NOT NULL DEFAULT '',
    first_name  TEXT NOT NULL DEFAULT '',
    last_name   TEXT NOT NULL DEFAULT '',
    vat_number  TEXT NOT NULL DEFAULT '',
    fiscal_code TEXT NOT NULL DEFAULT '',
    tax_regime  TEXT NOT NULL DEFAULT 'forfettario',
    albo_number TEXT NOT NULL DEFAULT '',
    albo_region TEXT NOT NULL DEFAULT '',
    address     TEXT NOT NULL DEFAULT '',
    city        TEXT NOT NULL DEFAULT '',
    province    TEXT NOT NULL DEFAULT '',
    zip_code    TEXT NOT NULL DEFAULT '',
    country     TEXT NOT NULL DEFAULT 'Italia',
    phone       TEXT NOT NULL DEFAULT '',
    pec_email   TEXT NOT NULL DEFAULT '',
    iban        TEXT NOT NULL DEFAULT '',
    coefficient REAL NOT NULL DEFAULT 78.0,
    is_psicoanalista INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS clients (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    client_type     TEXT NOT NULL DEFAULT 'persona_fisica',
    first_name      TEXT NOT NULL DEFAULT '',
    last_name       TEXT NOT NULL DEFAULT '',
    birth_date      TEXT,
    gender          TEXT DEFAULT 'M',
    fiscal_code     TEXT NOT NULL DEFAULT '',
    vat_number      TEXT DEFAULT '',
    address         TEXT NOT NULL DEFAULT '',
    city            TEXT NOT NULL DEFAULT '',
    province        TEXT NOT NULL DEFAULT '',
    zip_code        TEXT NOT NULL DEFAULT '',
    email           TEXT DEFAULT '',
    phone           TEXT NOT NULL DEFAULT '',
    notes           TEXT DEFAULT '',
    sts_authorization INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS services (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL,
    description   TEXT DEFAULT '',
    default_price REAL NOT NULL DEFAULT 0.0,
    vat_rate      REAL NOT NULL DEFAULT 0.0,
    is_active     INTEGER NOT NULL DEFAULT 1,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS invoices (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id        INTEGER NOT NULL REFERENCES clients(id),
    invoice_number   TEXT NOT NULL,
    year             INTEGER NOT NULL,
    issue_date       TEXT NOT NULL,
    due_date         TEXT,
    status           TEXT NOT NULL DEFAULT 'draft',
    payment_method   TEXT NOT NULL DEFAULT 'bonifico',
    notes            TEXT DEFAULT '',
    apply_enpap      INTEGER NOT NULL DEFAULT 1,
    contributo_enpap REAL NOT NULL DEFAULT 0.0,
    ritenuta_acconto REAL NOT NULL DEFAULT 0.0,
    marca_da_bollo   INTEGER NOT NULL DEFAULT 0,
    total_net        REAL NOT NULL DEFAULT 0.0,
    total_tax        REAL NOT NULL DEFAULT 0.0,
    total_gross      REAL NOT NULL DEFAULT 0.0,
    total_due        REAL NOT NULL DEFAULT 0.0,
    paid_date        TEXT,
    created_at       TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(year, invoice_number)
);
CREATE TABLE IF NOT EXISTS invoice_lines (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id  INTEGER NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    service_id  INTEGER REFERENCES services(id) ON DELETE SET NULL,
    description TEXT NOT NULL,
    quantity    INTEGER NOT NULL DEFAULT 1,
    unit_price  REAL NOT NULL DEFAULT 0.0,
    vat_rate    REAL NOT NULL DEFAULT 0.0,
    line_total  REAL NOT NULL DEFAULT 0.0
);
CREATE TABLE IF NOT EXISTS recurrence_groups (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id  INTEGER NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS appointments (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id           INTEGER NOT NULL REFERENCES clients(id),
    service_id          INTEGER REFERENCES services(id) ON DELETE SET NULL,
    date                TEXT NOT NULL,
    start_time          TEXT NOT NULL,
    end_time            TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'scheduled',
    notes               TEXT DEFAULT '',
    recurrence_group_id INTEGER REFERENCES recurrence_groups(id) ON DELETE SET NULL,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_invoices_year    ON invoices(year);
CREATE INDEX IF NOT EXISTS idx_invoices_status  ON invoices(status);
CREATE INDEX IF NOT EXISTS idx_invoices_client  ON invoices(client_id);
CREATE INDEX IF NOT EXISTS idx_appointments_date ON appointments(date);
";
