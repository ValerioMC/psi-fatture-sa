use serde::{Deserialize, Serialize};

/// How the Sistema TS identifies a document: issuer P.IVA, issue date and number.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TsDocumentId {
    pub vat_number: String,
    pub issue_date: String,
    pub number: String,
}
