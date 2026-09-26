use super::{TsDocumentId, TsExpenseItem};

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

    /// Paid before the invoice was issued: the Sistema TS wants it flagged.
    pub fn paid_in_advance(&self) -> bool {
        self.payment_date < self.id.issue_date
    }
}
