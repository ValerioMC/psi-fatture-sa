use serde::{Deserialize, Serialize};

/// How the Sistema TS identifies a document: issuer P.IVA, issue date and number.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TsDocumentId {
    pub vat_number: String,
    pub issue_date: String,
    pub number: String,
}

/// VAT treatment of one expense item: a rate, or the natura code of an exemption.
#[derive(Debug, Clone, PartialEq)]
pub enum TsVatTreatment {
    Rate(f64),
    Natura(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TsExpenseItem {
    pub amount: f64,
    pub vat: TsVatTreatment,
}

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
