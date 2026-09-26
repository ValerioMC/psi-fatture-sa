use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::app::repository::secret::SecretStore;
use crate::app::repository::ts::sistema_ts::SistemaTsGateway;

/// Application state shared across all Tauri commands.
pub struct AppState {
    pub db: DatabaseConnection,
    pub secrets: Arc<dyn SecretStore>,
    pub ts_gateway: Arc<dyn SistemaTsGateway>,
}
