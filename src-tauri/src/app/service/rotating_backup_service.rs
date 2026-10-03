//! The rotation of automatic copies beside the archive: one a day, plus any
//! taken by hand, the newest `KEEP` kept and the rest deleted. Names are the
//! only record, so nothing here touches the database's tables.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::NaiveDateTime;
use sea_orm::DatabaseConnection;

use crate::app::common::AppError;
use crate::app::model::backup::{BackupFile, BackupOverview, BackupReason};
use crate::app::repository::backup_repository;

pub const KEEP: usize = 10;
const PREFIX: &str = "psi-fatture-";
const EXTENSION: &str = ".db";
const STAMP: &str = "%Y%m%d-%H%M%S";
const DISPLAY: &str = "%Y-%m-%d %H:%M:%S";

/// Owns the backups folder. `writing` keeps a click on "Backup ora" and the
/// daily copy from writing at once; `last_failure` is shown until a copy succeeds.
pub struct RotatingBackups {
    database: PathBuf,
    folder: PathBuf,
    writing: tokio::sync::Mutex<()>,
    last_failure: Mutex<Option<String>>,
}

/// A backup's name taken apart; `sequence` orders copies taken in the same second.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedName {
    created_at: NaiveDateTime,
    sequence: u32,
    reason: BackupReason,
}

impl RotatingBackups {
    pub fn new(database: PathBuf, folder: PathBuf) -> Self {
        Self {
            database,
            folder,
            writing: tokio::sync::Mutex::new(()),
            last_failure: Mutex::new(None),
        }
    }

    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// A folder that cannot be read is reported in `last_failure`, never as a failed page.
    pub fn overview(&self) -> BackupOverview {
        let backups = self.list().unwrap_or_else(|error| {
            self.record_failure(&error);
            Vec::new()
        });
        BackupOverview {
            database_path: self.database.to_string_lossy().into_owned(),
            backups_folder: self.folder.to_string_lossy().into_owned(),
            keep: KEEP,
            backups,
            last_failure: self.last_failure.lock().expect("never poisoned").clone(),
        }
    }

    /// Newest first; files whose name this service did not hand out are ignored.
    pub fn list(&self) -> Result<Vec<BackupFile>, AppError> {
        let mut parsed: Vec<(ParsedName, BackupFile)> =
            backup_repository::list_files(&self.folder)?
                .into_iter()
                .filter_map(|stored| {
                    let name = parse_name(&stored.name)?;
                    let file = BackupFile {
                        created_at: name.created_at.format(DISPLAY).to_string(),
                        reason: name.reason,
                        name: stored.name,
                        path: stored.path.to_string_lossy().into_owned(),
                        size_bytes: stored.size_bytes,
                    };
                    Some((name, file))
                })
                .collect();
        parsed
            .sort_by(|(a, _), (b, _)| (b.created_at, b.sequence).cmp(&(a.created_at, a.sequence)));
        Ok(parsed.into_iter().map(|(_, file)| file).collect())
    }

    /// Writes a copy through a hidden staging file, so a failure never leaves half a backup, then prunes.
    pub async fn back_up(
        &self,
        db: &DatabaseConnection,
        reason: BackupReason,
        now: NaiveDateTime,
    ) -> Result<BackupFile, AppError> {
        let _writing = self.writing.lock().await;
        let outcome = self.write_copy(db, reason, now).await;
        match &outcome {
            Ok(file) => {
                *self.last_failure.lock().expect("never poisoned") = None;
                log::info!(target: "backup", path:% = file.path; "backup written");
            }
            Err(error) => self.record_failure(error),
        }
        outcome
    }

    /// The daily copy: due when no backup, automatic or manual, carries today's date.
    pub async fn back_up_if_due(
        &self,
        db: &DatabaseConnection,
        now: NaiveDateTime,
    ) -> Result<Option<BackupFile>, AppError> {
        let newest = self
            .list()
            .inspect_err(|error| self.record_failure(error))?;
        let taken_today = newest
            .first()
            .and_then(|file| NaiveDateTime::parse_from_str(&file.created_at, DISPLAY).ok())
            .is_some_and(|created| created.date() == now.date());
        if taken_today {
            return Ok(None);
        }
        self.back_up(db, BackupReason::Auto, now).await.map(Some)
    }

    async fn write_copy(
        &self,
        db: &DatabaseConnection,
        reason: BackupReason,
        now: NaiveDateTime,
    ) -> Result<BackupFile, AppError> {
        backup_repository::create_folder(&self.folder)?;
        let target = self.fresh_path(reason, now);
        let file_name = target.file_name().unwrap_or_default().to_string_lossy();
        let staging = self.folder.join(format!(".{file_name}.partial"));
        backup_repository::remove(&staging)?;
        if let Err(error) = backup_repository::copy_database_to(db, &staging).await {
            backup_repository::remove(&staging)?;
            return Err(error);
        }
        backup_repository::rename(&staging, &target)?;
        self.prune()?;
        let name = target.file_name().unwrap_or_default().to_string_lossy();
        self.list()?
            .into_iter()
            .find(|file| file.name == name)
            .ok_or_else(|| {
                AppError::External(format!(
                    "Il backup {} non è stato trovato dopo il salvataggio",
                    target.display()
                ))
            })
    }

    /// The first free sequence for this second, whatever the reason of the copies already in it.
    fn fresh_path(&self, reason: BackupReason, now: NaiveDateTime) -> PathBuf {
        let stamp = now.format(STAMP).to_string();
        let name_for = |sequence: u32, slug: &str| match sequence {
            0 => format!("{PREFIX}{stamp}-{slug}{EXTENSION}"),
            n => format!("{PREFIX}{stamp}-{n}-{slug}{EXTENSION}"),
        };
        let taken = |sequence: u32| {
            [BackupReason::Auto, BackupReason::Manual]
                .iter()
                .any(|other| self.folder.join(name_for(sequence, other.slug())).exists())
        };
        let sequence = (0..).find(|&sequence| !taken(sequence)).unwrap_or_default();
        self.folder.join(name_for(sequence, reason.slug()))
    }

    /// A copy that cannot be deleted is logged and left: the next prune tries again.
    fn prune(&self) -> Result<(), AppError> {
        for stale in self.list()?.iter().skip(KEEP) {
            if let Err(error) = backup_repository::remove(Path::new(&stale.path)) {
                log::warn!(target: "backup", error:% = error; "old backup not deleted");
            }
        }
        Ok(())
    }

    fn record_failure(&self, error: &AppError) {
        log::warn!(target: "backup", error:% = error; "backup failed");
        *self.last_failure.lock().expect("never poisoned") = Some(error.to_string());
    }
}

/// `psi-fatture-YYYYMMDD-HHMMSS[-N]-(auto|manual).db`, or None for any other name.
fn parse_name(name: &str) -> Option<ParsedName> {
    let inner = name.strip_prefix(PREFIX)?.strip_suffix(EXTENSION)?;
    let parts: Vec<&str> = inner.split('-').collect();
    let (date, time, sequence, reason) = match parts[..] {
        [date, time, reason] => (date, time, 0, reason),
        [date, time, sequence, reason] => (date, time, sequence.parse().ok()?, reason),
        _ => return None,
    };
    if date.len() != 8 || time.len() != 6 {
        return None;
    }
    let created_at = NaiveDateTime::parse_from_str(&format!("{date}-{time}"), STAMP).ok()?;
    Some(ParsedName {
        created_at,
        sequence,
        reason: BackupReason::of_slug(reason)?,
    })
}

#[cfg(test)]
#[path = "rotating_backup_service_test.rs"]
mod tests;
