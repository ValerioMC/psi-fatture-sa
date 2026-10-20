use crate::app::common::AppError;
use std::path::{Path, PathBuf};

/// Writes a document where the professional asked for it.
pub fn write(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    std::fs::write(path, bytes)
        .map_err(|e| AppError::External(format!("Salvataggio del file non riuscito: {e}")))
}

/// Writes a document into the app's folder of the system temp directory, replacing any
/// earlier preview of the same name, and returns where it went.
pub fn write_preview(file_name: &str, bytes: &[u8]) -> Result<PathBuf, AppError> {
    let folder = std::env::temp_dir().join("psi-fatture");
    std::fs::create_dir_all(&folder)
        .map_err(|e| AppError::External(format!("Cartella temporanea non disponibile: {e}")))?;
    let path = folder.join(file_name);
    write(&path, bytes)?;
    Ok(path)
}
