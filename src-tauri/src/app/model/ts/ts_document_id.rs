use serde::{Deserialize, Serialize};

/// How the Sistema TS identifies a document: issuer P.IVA, issue date and number.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TsDocumentId {
    pub vat_number: String,
    pub issue_date: String,
    pub number: String,
}

impl TsDocumentId {
    /// The id exists only once all three parts were stored, when the submission was claimed.
    pub fn from_stored(
        vat_number: Option<String>,
        issue_date: Option<String>,
        number: Option<String>,
    ) -> Option<TsDocumentId> {
        Some(TsDocumentId {
            vat_number: vat_number?,
            issue_date: issue_date?,
            number: number?,
        })
    }
}
