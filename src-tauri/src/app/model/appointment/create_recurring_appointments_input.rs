use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecurringAppointmentsInput {
    pub client_id: i64,
    pub service_id: Option<i64>,
    pub dates: Vec<String>,
    pub start_time: String,
    pub end_time: String,
    pub notes: String,
}
