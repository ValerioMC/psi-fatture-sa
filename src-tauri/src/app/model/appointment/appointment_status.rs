use serde::{Deserialize, Serialize};

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
