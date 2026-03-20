use tauri::State;

use crate::app::model::dashboard::DashboardData;
use crate::app::service::dashboard_service;
use crate::AppState;

/// Returns aggregated dashboard analytics for the given year.
#[tauri::command]
pub async fn get_dashboard(state: State<'_, AppState>, year: i64) -> Result<DashboardData, String> {
    dashboard_service::get(&state.db, year).await
}
