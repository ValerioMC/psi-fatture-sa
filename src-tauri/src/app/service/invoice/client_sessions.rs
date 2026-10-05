use std::collections::BTreeMap;

use crate::app::entity::service as services;
use crate::app::model::appointment::Appointment;
use crate::app::model::invoice::InvoiceLineInput;

/// What a line says when its sessions have no service: it is priced at zero,
/// so monthly invoicing refuses it until a service is assigned.
const UNASSIGNED_SERVICE_NAME: &str = "Seduta";

/// One patient's unbilled sessions of a month: what a monthly invoice is built from.
pub(super) struct ClientSessions {
    pub client_id: i64,
    pub client_name: String,
    pub sessions: Vec<Appointment>,
}

impl ClientSessions {
    /// One group per patient, in patient id order.
    pub fn group_by_client(appointments: Vec<Appointment>) -> Vec<ClientSessions> {
        let mut by_client: BTreeMap<i64, ClientSessions> = BTreeMap::new();
        for appointment in appointments {
            by_client
                .entry(appointment.client_id)
                .or_insert_with(|| ClientSessions {
                    client_id: appointment.client_id,
                    client_name: appointment.client_name.clone(),
                    sessions: Vec::new(),
                })
                .sessions
                .push(appointment);
        }
        by_client.into_values().collect()
    }

    pub fn appointment_ids(&self) -> Vec<i64> {
        self.sessions.iter().map(|session| session.id).collect()
    }

    /// One line per service, priced at the service's default and listing the session dates.
    pub fn invoice_lines(
        &self,
        services_by_id: &BTreeMap<i64, services::Model>,
    ) -> Vec<InvoiceLineInput> {
        let mut dates_by_service: BTreeMap<Option<i64>, Vec<String>> = BTreeMap::new();
        for session in &self.sessions {
            dates_by_service
                .entry(session.service_id)
                .or_default()
                .push(day_and_month(&session.date));
        }
        dates_by_service
            .into_iter()
            .map(|(service_id, dates)| {
                let service = service_id.and_then(|id| services_by_id.get(&id));
                service_line(service_id, service, &dates)
            })
            .collect()
    }
}

fn service_line(
    service_id: Option<i64>,
    service: Option<&services::Model>,
    dates: &[String],
) -> InvoiceLineInput {
    let (service_name, unit_price, vat_rate) = match service {
        Some(service) => (
            service.name.as_str(),
            service.default_price,
            service.vat_rate,
        ),
        None => (UNASSIGNED_SERVICE_NAME, 0.0, 0.0),
    };
    InvoiceLineInput {
        service_id,
        description: line_description(service_name, dates),
        quantity: dates.len() as i64,
        unit_price,
        vat_rate,
        amount_override: None,
    }
}

fn line_description(service_name: &str, dates: &[String]) -> String {
    let listed_dates = dates.join(", ");
    match dates.len() {
        1 => format!("{service_name} del {listed_dates}"),
        count => format!("{service_name} — {count} sedute ({listed_dates})"),
    }
}

/// `2026-03-05` as `05/03`; a date that does not parse is shown as stored.
fn day_and_month(iso_date: &str) -> String {
    chrono::NaiveDate::parse_from_str(iso_date, "%Y-%m-%d")
        .map(|date| date.format("%d/%m").to_string())
        .unwrap_or_else(|_| iso_date.to_owned())
}

#[cfg(test)]
#[path = "client_sessions_test.rs"]
mod tests;
