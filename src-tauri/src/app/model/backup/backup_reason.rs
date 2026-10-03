use serde::Serialize;

/// Why a copy in the rotation exists: the daily schedule or a click on "Backup ora".
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackupReason {
    Auto,
    Manual,
}

impl BackupReason {
    /// The word the reason takes in a backup's file name.
    pub fn slug(self) -> &'static str {
        match self {
            BackupReason::Auto => "auto",
            BackupReason::Manual => "manual",
        }
    }

    pub fn of_slug(slug: &str) -> Option<Self> {
        match slug {
            "auto" => Some(BackupReason::Auto),
            "manual" => Some(BackupReason::Manual),
            _ => None,
        }
    }
}
