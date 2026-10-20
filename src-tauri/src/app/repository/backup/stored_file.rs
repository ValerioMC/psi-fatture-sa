use std::path::PathBuf;

/// A regular file found in the backups folder, before its name is checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredFile {
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
}
