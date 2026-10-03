use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::app::repository::email::MailGateway;
use crate::app::repository::release::ReleaseGateway;
use crate::app::repository::secret::SecretStore;
use crate::app::repository::ts::sistema_ts::SistemaTsGateway;
use crate::app::service::rotating_backup_service::RotatingBackups;

/// Application state shared across all Tauri commands.
pub struct AppState {
    pub db: DatabaseConnection,
    pub secrets: Arc<dyn SecretStore>,
    pub ts_gateway: Arc<dyn SistemaTsGateway>,
    pub mail_gateway: Arc<dyn MailGateway>,
    pub backups: Arc<RotatingBackups>,
    pub release_gateway: Arc<dyn ReleaseGateway>,
}
