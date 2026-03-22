pub mod appointment;
pub mod create_appointment_input;
pub mod create_recurring_appointments_input;
pub mod update_appointment_input;

pub use appointment::{Appointment, AppointmentStatus};
pub use create_appointment_input::CreateAppointmentInput;
pub use create_recurring_appointments_input::CreateRecurringAppointmentsInput;
pub use update_appointment_input::UpdateAppointmentInput;
