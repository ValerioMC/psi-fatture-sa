use crate::app::model::client::Client;
use crate::app::model::config::ProfessionalConfig;
use crate::app::model::invoice::Invoice;

/// Everything printed on an invoice: the invoice, who it is addressed to and who issues it.
pub struct InvoiceDocument {
    pub invoice: Invoice,
    pub client: Client,
    pub config: ProfessionalConfig,
}
