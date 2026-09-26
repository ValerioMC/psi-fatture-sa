//! Background sender for the Sistema TS queue. It requeues calls a previous
//! run left in flight, then passes over the queue every minute, so a
//! submission goes out as soon as the connection comes back.

use std::sync::Arc;
use std::time::Duration;

use sea_orm::DatabaseConnection;

use crate::app::repository::secret_store::SecretStore;
use crate::app::repository::sistema_ts::gateway::SistemaTsGateway;
use crate::app::service::{ts_dispatch_service, ts_transition_service};

const PASS_INTERVAL: Duration = Duration::from_secs(60);

pub fn spawn(
    db: DatabaseConnection,
    secrets: Arc<dyn SecretStore>,
    gateway: Arc<dyn SistemaTsGateway>,
) {
    tauri::async_runtime::spawn(async move {
        let started = chrono::Utc::now().naive_utc();
        if let Err(error) = ts_transition_service::requeue_interrupted(&db, started).await {
            log::warn!(target: "sistema_ts", error:% = error; "requeue of interrupted calls failed");
        }
        loop {
            let now = chrono::Utc::now().naive_utc();
            match ts_dispatch_service::dispatch_due(&db, secrets.as_ref(), gateway.as_ref(), now)
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
