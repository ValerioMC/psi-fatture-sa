use sea_orm::FromQueryResult;

use crate::app::model::invoice::{Invoice, InvoiceLine, InvoiceStatus, PaymentMethod};

/// An `invoices` row joined with its client's name, before the lines are attached.
#[derive(FromQueryResult)]
pub(super) struct InvoiceRow {
    pub(super) id: i64,
    client_id: i64,
    client_name: String,
    invoice_number: String,
    year: i64,
    issue_date: String,
    due_date: Option<String>,
    status: String,
    payment_method: String,
    notes: Option<String>,
    apply_enpap: i32,
    contributo_enpap: f64,
    ritenuta_acconto: f64,
    marca_da_bollo: i32,
    total_net: f64,
    total_tax: f64,
    total_gross: f64,
    total_due: f64,
    paid_date: Option<String>,
    created_at: String,
    updated_at: String,
}

impl InvoiceRow {
    pub(super) fn into_invoice(self, lines: Vec<InvoiceLine>) -> Invoice {
        Invoice {
            id: self.id,
            client_id: self.client_id,
            client_name: self.client_name,
            invoice_number: self.invoice_number,
            year: self.year,
            issue_date: self.issue_date,
            due_date: self.due_date,
            status: InvoiceStatus::from(self.status),
            payment_method: PaymentMethod::from(self.payment_method),
            notes: self.notes.unwrap_or_default(),
            apply_enpap: self.apply_enpap != 0,
            contributo_enpap: self.contributo_enpap,
            ritenuta_acconto: self.ritenuta_acconto,
            marca_da_bollo: self.marca_da_bollo != 0,
            total_net: self.total_net,
            total_tax: self.total_tax,
            total_gross: self.total_gross,
            total_due: self.total_due,
            paid_date: self.paid_date,
            lines,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
