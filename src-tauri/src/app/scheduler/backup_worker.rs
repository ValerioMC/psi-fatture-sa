//! Background taker of the daily backup. It looks a few seconds after start,
//! then every hour, so an app left open overnight still gets the next day's copy.

use std::sync::Arc;
use std::time::Duration;

use sea_orm::DatabaseConnection;

use crate::app::service::rotating_backup_service::RotatingBackups;

const FIRST_CHECK_AFTER: Duration = Duration::from_secs(5);
const CHECK_EVERY: Duration = Duration::from_secs(3600);

pub fn spawn(db: DatabaseConnection, backups: Arc<RotatingBackups>) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_CHECK_AFTER).await;
        loop {
            let now = chrono::Local::now().naive_local();
            if let Err(error) = backups.back_up_if_due(&db, now).await {
                log::warn!(target: "backup", error:% = error; "daily backup failed");
            }
            tokio::time::sleep(CHECK_EVERY).await;
        }
    });
}
