//! An in-memory database with the schema migrated and no rows.

use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

pub async fn setup() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    crate::migration::Migrator::up(&db, None).await.unwrap();
    db
}
