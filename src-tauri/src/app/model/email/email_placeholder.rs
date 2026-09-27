use super::EmailPlaceholderInfo;

/// A value a template can name between braces, `{paziente}`. The list is closed:
/// saving a template with any other name is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailPlaceholder {
    Patient,
    PatientFirstName,
    InvoiceNumber,
    InvoiceDate,
    Amount,
    DueDate,
    Professional,
}

impl EmailPlaceholder {
    pub const ALL: [EmailPlaceholder; 7] = [
        EmailPlaceholder::Patient,
        EmailPlaceholder::PatientFirstName,
        EmailPlaceholder::InvoiceNumber,
        EmailPlaceholder::InvoiceDate,
        EmailPlaceholder::Amount,
        EmailPlaceholder::DueDate,
        EmailPlaceholder::Professional,
    ];

    pub fn key(&self) -> &'static str {
        match self {
            EmailPlaceholder::Patient => "paziente",
            EmailPlaceholder::PatientFirstName => "nome_paziente",
            EmailPlaceholder::InvoiceNumber => "numero_fattura",
            EmailPlaceholder::InvoiceDate => "data_fattura",
            EmailPlaceholder::Amount => "importo",
            EmailPlaceholder::DueDate => "scadenza",
            EmailPlaceholder::Professional => "professionista",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            EmailPlaceholder::Patient => "Paziente",
            EmailPlaceholder::PatientFirstName => "Nome del paziente",
            EmailPlaceholder::InvoiceNumber => "Numero fattura",
            EmailPlaceholder::InvoiceDate => "Data fattura",
            EmailPlaceholder::Amount => "Importo dovuto",
            EmailPlaceholder::DueDate => "Scadenza",
            EmailPlaceholder::Professional => "Il tuo nome",
        }
    }

    pub fn parse(key: &str) -> Option<EmailPlaceholder> {
        EmailPlaceholder::ALL.into_iter().find(|p| p.key() == key)
    }

    pub fn describe(&self) -> EmailPlaceholderInfo {
        EmailPlaceholderInfo {
            key: self.key(),
            label: self.label(),
        }
    }
}
