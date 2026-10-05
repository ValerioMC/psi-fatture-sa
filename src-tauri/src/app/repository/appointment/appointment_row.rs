use sea_orm::FromQueryResult;

use crate::app::model::appointment::{Appointment, AppointmentStatus};

/// An `appointments` row joined with its client's and service's names.
#[derive(FromQueryResult)]
pub(super) struct AppointmentRow {
    id: i64,
    client_id: i64,
    service_id: Option<i64>,
    date: String,
    start_time: String,
    end_time: String,
    status: String,
    notes: Option<String>,
    recurrence_group_id: Option<i64>,
    invoice_id: Option<i64>,
    created_at: String,
    updated_at: String,
    client_name: String,
    service_name: Option<String>,
}

impl AppointmentRow {
    pub(super) fn into_appointment(self) -> Appointment {
        Appointment {
            id: self.id,
            client_id: self.client_id,
            client_name: self.client_name,
            service_id: self.service_id,
            service_name: self.service_name,
            date: self.date,
            start_time: self.start_time,
            end_time: self.end_time,
            status: AppointmentStatus::from(self.status),
            notes: self.notes.unwrap_or_default(),
            recurrence_group_id: self.recurrence_group_id,
            invoice_id: self.invoice_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
