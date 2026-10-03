use serde::Serialize;

use super::BackupReason;

/// One copy of the archive in the backups folder. `created_at` is local time,
/// `YYYY-MM-DD HH:MM:SS`, read from the file name.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BackupFile {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub reason: BackupReason,
}
