use std::path::Path;

use sea_orm::{ConnectionTrait, Database, Statement};

use super::StoredFile;
use crate::app::common::AppError;

/// Writes a consistent copy of the live database to `target`, which must not exist yet.
/// `VACUUM INTO` reads through the WAL, so the copy includes every committed change.
pub async fn copy_database_to(db: &impl ConnectionTrait, target: &Path) -> Result<(), AppError> {
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "VACUUM INTO ?",
        [target.to_string_lossy().to_string().into()],
    ))
    .await?;
    Ok(())
}

pub fn create_folder(folder: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(folder).map_err(|e| {
        AppError::External(format!(
            "Impossibile creare la cartella dei backup {}: {e}",
            folder.display()
        ))
    })
}

/// The regular files in `folder`; a folder not created yet holds none.
pub fn list_files(folder: &Path) -> Result<Vec<StoredFile>, AppError> {
    if !folder.is_dir() {
        return Ok(Vec::new());
    }
    let entries = std::fs::read_dir(folder).map_err(|e| {
        AppError::External(format!(
            "Impossibile leggere la cartella dei backup {}: {e}",
            folder.display()
        ))
    })?;
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
            log::warn!(target: "backup", path:? = entry.path(); "unreadable entry skipped");
            continue;
        };
        if metadata.is_file() {
            files.push(StoredFile {
                path: entry.path(),
                name: entry.file_name().to_string_lossy().into_owned(),
                size_bytes: metadata.len(),
            });
        }
    }
    Ok(files)
}

pub fn rename(from: &Path, to: &Path) -> Result<(), AppError> {
    std::fs::rename(from, to)
        .map_err(|e| AppError::External(format!("Salvataggio del backup non riuscito: {e}")))
}

/// Deletes `path`; a file already gone is not an error.
pub fn remove(path: &Path) -> Result<(), AppError> {
    remove_if_present(path)
        .map_err(|e| AppError::External(format!("Impossibile eliminare {}: {e}", path.display())))
}

/// Deletes the partial copy a failed export left behind, before a new one is written.
pub fn clear_staging(path: &Path) -> Result<(), AppError> {
    remove_if_present(path)
        .map_err(|e| AppError::External(format!("Impossibile preparare il backup: {e}")))
}

fn remove_if_present(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

/// The migrations a backup has applied, read after SQLite's own integrity check.
/// A file that is not a readable archive is `Invalid`, whatever went wrong inside it.
pub async fn read_applied_migrations(archive: &Path) -> Result<Vec<String>, AppError> {
    read_migration_versions(archive).await.map_err(|error| {
        log::warn!(target: "backup", path:? = archive, error:% = error; "backup not readable");
        AppError::Invalid("Il backup scelto non è un archivio di PSI Fatture leggibile".to_string())
    })
}

async fn read_migration_versions(archive: &Path) -> Result<Vec<String>, sea_orm::DbErr> {
    let connection = Database::connect(format!("sqlite://{}?mode=ro", archive.display())).await?;
    let sqlite = sea_orm::DatabaseBackend::Sqlite;
    let check = connection
        .query_one(Statement::from_string(sqlite, "PRAGMA integrity_check"))
        .await?
        .map(|row| row.try_get::<String>("", "integrity_check"))
        .transpose()?;
    if check.as_deref() != Some("ok") {
        return Err(sea_orm::DbErr::Custom("integrity check failed".to_string()));
    }
    let rows = connection
        .query_all(Statement::from_string(
            sqlite,
            "SELECT version FROM seaql_migrations",
        ))
        .await?;
    let versions = rows
        .iter()
        .map(|row| row.try_get::<String>("", "version"))
        .collect::<Result<Vec<_>, _>>()?;
    connection.close().await?;
    Ok(versions)
}

pub fn copy_file(from: &Path, to: &Path) -> Result<(), AppError> {
    std::fs::copy(from, to)
        .map(|_| ())
        .map_err(|e| AppError::External(format!("Impossibile preparare il ripristino: {e}")))
}

/// Replaces the database with the file a restore staged, dropping the old database's
/// `-wal` and `-shm` files, which belong to the replaced data. Without a staged file it does nothing.
pub fn apply_staged_restore(database: &Path, staged: &Path) -> Result<(), AppError> {
    if !staged.exists() {
        return Ok(());
    }
    for suffix in ["-wal", "-shm"] {
        let mut name = database.file_name().unwrap_or_default().to_os_string();
        name.push(suffix);
        remove(&database.with_file_name(name))?;
    }
    std::fs::rename(staged, database)
        .map_err(|e| AppError::External(format!("Ripristino del backup non riuscito: {e}")))
}
