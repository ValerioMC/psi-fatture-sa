//! Database initialization: SeaORM connection pool + migrations.

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr, Statement};
use sea_orm_migration::MigratorTrait;
use std::path::PathBuf;

use crate::migration::Migrator;

/// Returns the path to the SQLite database file.
pub fn db_path() -> PathBuf {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("psi-fatture-sa");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir.join("database.db")
}

/// Initializes the SeaORM connection pool and runs all pending migrations.
pub async fn init_db() -> Result<DatabaseConnection, DbErr> {
    let db_url = format!("sqlite://{}?mode=rwc", db_path().display());

    let mut opts = ConnectOptions::new(db_url);
    opts.sqlx_logging(false);

    let db = Database::connect(opts).await?;

    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;".to_owned(),
    ))
    .await?;

    Migrator::up(&db, None).await?;

    Ok(db)
}
