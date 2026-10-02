use sha2::{Digest, Sha256};

use super::{TsDocumentId, TsExpenseItem, TsVatTreatment};

/// One health expense as the Sistema TS records it. `citizen_fiscal_code`
/// is absent exactly when the patient opposed the transmission.
#[derive(Debug, Clone, PartialEq)]
pub struct TsExpenseDocument {
    pub id: TsDocumentId,
    pub payment_date: String,
    pub citizen_fiscal_code: Option<String>,
    pub items: Vec<TsExpenseItem>,
    pub traced_payment: bool,
}

impl TsExpenseDocument {
    pub fn opposed(&self) -> bool {
        self.citizen_fiscal_code.is_none()
    }

    /// A digest of what the Sistema TS records about the expense, id excluded,
    /// so two documents for the same invoice compare by content.
    pub fn fingerprint(&self) -> String {
        let items: Vec<String> = self
            .items
            .iter()
            .map(|item| match &item.vat {
                TsVatTreatment::Rate(rate) => format!("{:.2}@R{rate:.2}", item.amount),
                TsVatTreatment::Natura(code) => format!("{:.2}@N{code}", item.amount),
            })
            .collect();
        let canonical = format!(
            "{}|{}|{}|{}",
            self.payment_date,
            self.citizen_fiscal_code.as_deref().unwrap_or("-"),
            self.traced_payment,
            items.join(";")
        );
        format!("{:x}", Sha256::digest(canonical.as_bytes()))
    }

    /// Paid before the invoice was issued: the Sistema TS wants it flagged.
    pub fn paid_in_advance(&self) -> bool {
        self.payment_date < self.id.issue_date
    }
}
