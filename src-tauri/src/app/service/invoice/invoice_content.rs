use crate::app::model::invoice::{
    CreateInvoiceInput, InvoiceLineInput, InvoiceStatus, PaymentMethod, UpdateInvoiceInput,
};

/// What the user wrote on an invoice, the same whether it is being created or
/// updated: the number, the id and the quantity visibility are resolved apart.
pub(super) struct InvoiceContent<'a> {
    pub client_id: i64,
    pub issue_date: &'a str,
    pub due_date: Option<&'a str>,
    pub status: &'a InvoiceStatus,
    pub payment_method: &'a PaymentMethod,
    pub notes: &'a str,
    pub apply_enpap: bool,
    pub paid_date: Option<&'a str>,
    pub lines: &'a [InvoiceLineInput],
}

impl<'a> From<&'a CreateInvoiceInput> for InvoiceContent<'a> {
    fn from(input: &'a CreateInvoiceInput) -> Self {
        Self {
            client_id: input.client_id,
            issue_date: &input.issue_date,
            due_date: input.due_date.as_deref(),
            status: &input.status,
            payment_method: &input.payment_method,
            notes: &input.notes,
            apply_enpap: input.apply_enpap,
            paid_date: None,
            lines: &input.lines,
        }
    }
}

impl<'a> From<&'a UpdateInvoiceInput> for InvoiceContent<'a> {
    fn from(input: &'a UpdateInvoiceInput) -> Self {
        Self {
            client_id: input.client_id,
            issue_date: &input.issue_date,
            due_date: input.due_date.as_deref(),
            status: &input.status,
            payment_method: &input.payment_method,
            notes: &input.notes,
            apply_enpap: input.apply_enpap,
            paid_date: input.paid_date.as_deref(),
            lines: &input.lines,
        }
    }
}
