use crate::app::model::client::ClientType;
use crate::app::model::email::EmailPlaceholder;
use crate::app::service::italian_format::{currency, date_long};
use crate::app::service::pdf::InvoiceDocument;

/// The text each placeholder stands for, for one invoice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailValues {
    pub patient: String,
    pub patient_first_name: String,
    pub invoice_number: String,
    pub invoice_date: String,
    pub amount: String,
    pub due_date: String,
    pub professional: String,
}

impl EmailValues {
    pub fn of(document: &InvoiceDocument) -> Self {
        let client = &document.client;
        let config = &document.config;
        let invoice = &document.invoice;
        let patient = match client.client_type {
            ClientType::Azienda => client.last_name.trim().to_string(),
            ClientType::PersonaFisica => {
                format!("{} {}", client.first_name.trim(), client.last_name.trim())
                    .trim()
                    .to_string()
            }
        };
        let patient_first_name = match client.first_name.trim() {
            "" => patient.clone(),
            first => first.to_string(),
        };
        EmailValues {
            patient,
            patient_first_name,
            invoice_number: format!("{}/{}", invoice.invoice_number, invoice.year),
            invoice_date: date_long(&invoice.issue_date),
            amount: currency(invoice.total_due),
            due_date: invoice
                .due_date
                .as_deref()
                .map(date_long)
                .unwrap_or_default(),
            professional: format!(
                "{} {} {}",
                config.title, config.first_name, config.last_name
            )
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
        }
    }

    pub fn get(&self, placeholder: EmailPlaceholder) -> String {
        match placeholder {
            EmailPlaceholder::Patient => self.patient.clone(),
            EmailPlaceholder::PatientFirstName => self.patient_first_name.clone(),
            EmailPlaceholder::InvoiceNumber => self.invoice_number.clone(),
            EmailPlaceholder::InvoiceDate => self.invoice_date.clone(),
            EmailPlaceholder::Amount => self.amount.clone(),
            EmailPlaceholder::DueDate => self.due_date.clone(),
            EmailPlaceholder::Professional => self.professional.clone(),
        }
    }
}
