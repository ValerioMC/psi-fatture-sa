use serde::{Deserialize, Serialize};

use super::appointment::AppointmentStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAppointmentInput {
    pub client_id: i64,
    pub service_id: Option<i64>,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
    pub status: AppointmentStatus,
    pub notes: String,
    pub recurrence_group_id: Option<i64>,
}
