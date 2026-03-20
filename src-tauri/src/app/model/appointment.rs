use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appointment {
    pub id: i64,
    pub client_id: i64,
    pub client_name: String,
    pub service_id: Option<i64>,
    pub service_name: Option<String>,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
    pub status: AppointmentStatus,
    pub notes: String,
    pub recurrence_group_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAppointmentInput {
    pub id: i64,
    pub client_id: i64,
    pub service_id: Option<i64>,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
    pub status: AppointmentStatus,
    pub notes: String,
    pub recurrence_group_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecurringAppointmentsInput {
    pub client_id: i64,
    pub service_id: Option<i64>,
    pub dates: Vec<String>,
    pub start_time: String,
    pub end_time: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppointmentStatus {
    Scheduled,
    Completed,
    Cancelled,
}

impl AppointmentStatus {
    pub fn as_str(&self) -> &str {
        match self {
            AppointmentStatus::Scheduled => "scheduled",
            AppointmentStatus::Completed => "completed",
            AppointmentStatus::Cancelled => "cancelled",
        }
    }
}

impl From<String> for AppointmentStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "completed" => AppointmentStatus::Completed,
            "cancelled" => AppointmentStatus::Cancelled,
            _ => AppointmentStatus::Scheduled,
        }
    }
}
