use crate::app::model::invoice::{Invoice, UpdateInvoiceInput};

/// An update whose input passed validation, with the invoice it changes and the number
/// and year it gives it. Only `invoice_service::check_update` builds one, so applying an
/// update cannot skip the checks.
pub struct CheckedUpdate {
    pub(super) input: UpdateInvoiceInput,
    pub(super) current: Invoice,
    pub(super) number: i64,
    pub(super) year: i64,
}

impl CheckedUpdate {
    pub fn input(&self) -> &UpdateInvoiceInput {
        &self.input
    }

    pub fn current(&self) -> &Invoice {
        &self.current
    }

    pub fn number(&self) -> i64 {
        self.number
    }
}
