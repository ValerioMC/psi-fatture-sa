use std::path::Path;

use sea_orm::{ConnectionTrait, Statement};

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
