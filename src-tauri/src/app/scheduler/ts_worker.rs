//! Background sender for the Sistema TS queue. It requeues calls a previous
//! run left in flight, then passes over the queue every minute, so a
//! submission goes out as soon as the connection comes back. Its progress
//! reaches the window like a manual send's, so open views stay current.

use std::sync::Arc;
use std::time::Duration;

use sea_orm::DatabaseConnection;
use tauri::AppHandle;

use crate::app::common::Clock;
use crate::app::controller::ts::ts_progress_event;
use crate::app::repository::secret::SecretStore;
use crate::app::repository::ts::sistema_ts::SistemaTsGateway;
use crate::app::service::ts::{ts_dispatch_service, ts_transition_service};

const PASS_INTERVAL: Duration = Duration::from_secs(60);

pub fn spawn(
    app: AppHandle,
    db: DatabaseConnection,
    clock: Arc<dyn Clock>,
    secrets: Arc<dyn SecretStore>,
    gateway: Arc<dyn SistemaTsGateway>,
) {
    tauri::async_runtime::spawn(async move {
        let report = ts_progress_event::emitter(app);
        if let Err(error) = ts_transition_service::requeue_interrupted(&db, clock.utc_now()).await {
            log::warn!(target: "sistema_ts", error:% = error; "requeue of interrupted calls failed");
        }
        loop {
            match ts_dispatch_service::dispatch_due(
                &db,
                secrets.as_ref(),
                gateway.as_ref(),
                &report,
                clock.utc_now(),
            )
            .await
            {
                Ok(summary) if summary.blocked.is_some() => {
                    log::info!(target: "sistema_ts", blocked:? = summary.blocked; "queue pass blocked");
                }
                Ok(_) => {}
                Err(error) => {
                    log::warn!(target: "sistema_ts", error:% = error; "queue pass failed");
                }
            }
            tokio::time::sleep(PASS_INTERVAL).await;
        }
    });
}
