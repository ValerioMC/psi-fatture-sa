use serde::Serialize;

use super::BackupFile;

/// What the "Dati e backup" page shows: where the archive lives, where its copies go,
/// the copies newest first and the last automatic backup that failed, if any.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BackupOverview {
    pub database_path: String,
    pub backups_folder: String,
    pub keep: usize,
    pub backups: Vec<BackupFile>,
    pub last_failure: Option<String>,
}
