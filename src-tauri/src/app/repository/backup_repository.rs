use std::path::Path;

use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

use crate::app::common::AppError;

/// Writes a consistent copy of the live database to `target`, which must not exist yet.
/// `VACUUM INTO` reads through the WAL, so the copy includes every committed change.
pub async fn copy_database_to(db: &DatabaseConnection, target: &Path) -> Result<(), AppError> {
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "VACUUM INTO ?",
        [target.to_string_lossy().to_string().into()],
    ))
    .await?;
    Ok(())
}
