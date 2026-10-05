use crate::app::model::appointment::AppointmentStatus;

use super::*;

fn session(id: i64, client_id: i64, service_id: Option<i64>, date: &str) -> Appointment {
    Appointment {
        id,
        client_id,
        client_name: format!("Paziente {client_id}"),
        service_id,
        service_name: None,
        date: date.to_string(),
        start_time: "10:00".to_string(),
        end_time: "11:00".to_string(),
        status: AppointmentStatus::Completed,
        notes: String::new(),
        recurrence_group_id: None,
        invoice_id: None,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

fn colloquio() -> BTreeMap<i64, services::Model> {
    let service = services::Model {
        id: 1,
        name: "Colloquio".to_string(),
        description: String::new(),
        default_price: 70.0,
        vat_rate: 0.0,
        is_active: 1,
        created_at: String::new(),
        updated_at: String::new(),
    };
    BTreeMap::from([(1, service)])
}

#[test]
fn groups_sessions_by_patient_in_id_order() {
    let groups = ClientSessions::group_by_client(vec![
        session(1, 2, Some(1), "2026-03-02"),
        session(2, 1, Some(1), "2026-03-03"),
        session(3, 2, Some(1), "2026-03-09"),
    ]);

    let clients: Vec<i64> = groups.iter().map(|group| group.client_id).collect();
    assert_eq!(clients, vec![1, 2]);
    assert_eq!(groups[1].appointment_ids(), vec![1, 3]);
    assert_eq!(groups[1].client_name, "Paziente 2");
}

#[test]
fn one_session_names_its_date() {
    let groups = ClientSessions::group_by_client(vec![session(1, 1, Some(1), "2026-03-05")]);

    let lines = groups[0].invoice_lines(&colloquio());

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].description, "Colloquio del 05/03");
    assert_eq!(lines[0].quantity, 1);
    assert_eq!(lines[0].unit_price, 70.0);
}

#[test]
fn sessions_of_one_service_share_a_line() {
    let groups = ClientSessions::group_by_client(vec![
        session(1, 1, Some(1), "2026-03-05"),
        session(2, 1, Some(1), "2026-03-12"),
    ]);

    let lines = groups[0].invoice_lines(&colloquio());

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].description, "Colloquio — 2 sedute (05/03, 12/03)");
    assert_eq!(lines[0].quantity, 2);
}

#[test]
fn a_session_without_service_gets_its_own_free_line() {
    let groups = ClientSessions::group_by_client(vec![
        session(1, 1, Some(1), "2026-03-05"),
        session(2, 1, None, "2026-03-12"),
    ]);

    let lines = groups[0].invoice_lines(&colloquio());

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].service_id, None);
    assert_eq!(lines[0].description, "Seduta del 12/03");
    assert_eq!(lines[0].unit_price, 0.0);
}
