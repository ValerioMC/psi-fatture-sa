//! File operations on `.app` bundles and the update's scratch folder. A bundle is
//! a directory, so removing one removes its whole tree.

use std::path::{Path, PathBuf};

use crate::app::common::AppError;

/// The first `.app` directory directly inside `folder`.
pub fn find_app_in(folder: &Path) -> Result<PathBuf, AppError> {
    let entries = std::fs::read_dir(folder)
        .map_err(|e| AppError::External(format!("Immagine disco non leggibile: {e}")))?;
    entries
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "app") && path.is_dir())
        .ok_or_else(|| {
            AppError::External("L'immagine disco non contiene l'app PSI Fatture".to_string())
        })
}

/// Leaves `folder` existing and empty.
pub fn reset_folder(folder: &Path) -> Result<(), AppError> {
    remove_tree(folder)?;
    std::fs::create_dir_all(folder)
        .map_err(|e| AppError::External(format!("Cartella {} non creabile: {e}", folder.display())))
}

/// Removes a file or directory tree; one already gone is not an error.
pub fn remove_tree(path: &Path) -> Result<(), AppError> {
    let outcome = match std::fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => Err(e),
        Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(path),
        Ok(_) => std::fs::remove_file(path),
    };
    outcome.map_err(|e| AppError::External(format!("{} non eliminabile: {e}", path.display())))
}

pub fn rename(from: &Path, to: &Path) -> Result<(), AppError> {
    std::fs::rename(from, to).map_err(|e| {
        AppError::External(format!(
            "Impossibile spostare {} in {}: {e}",
            from.display(),
            to.display()
        ))
    })
}
