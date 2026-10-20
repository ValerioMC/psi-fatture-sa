//! Backups of the whole archive: one SQLite file holding patients, invoices,
//! agenda and settings. Credentials stay out: they are sealed to this machine.

use std::path::{Path, PathBuf};

use sea_orm::DatabaseConnection;

use crate::app::common::{AppError, Clock};
use crate::app::repository::backup::backup_repository;

/// Copies the database to `target`, replacing a file the user chose to overwrite.
/// The copy lands beside the target first, so a failure never leaves half a backup.
pub async fn export(db: &DatabaseConnection, target: &Path) -> Result<(), AppError> {
    if target.as_os_str().is_empty() {
        return Err(AppError::Invalid(
            "Scegli dove salvare il backup".to_string(),
        ));
    }
    let staging = staging_path(target);
    backup_repository::clear_staging(&staging)?;
    backup_repository::copy_database_to(db, &staging).await?;
    backup_repository::rename(&staging, target)
}

/// The name the save dialog proposes: "PSI-Fatture-backup-2026-10-02.db".
pub fn file_name(clock: &dyn Clock) -> String {
    format!("PSI-Fatture-backup-{}.db", clock.today())
}

fn staging_path(target: &Path) -> PathBuf {
    let mut name = target.file_name().unwrap_or_default().to_os_string();
    name.push(".partial");
    target.with_file_name(name)
}

#[cfg(test)]
#[path = "backup_service_test.rs"]
mod tests;
