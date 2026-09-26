use serde::Serialize;

use super::ts_document::TsDocumentId;
use super::ts_response::TsMessage;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TsExpenseTotal {
    pub expense_type: String,
    pub amount: f64,
}

/// A document as the Sistema TS holds it, read back through the point query.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TsRemoteDocument {
    pub id: TsDocumentId,
    pub payment_date: Option<String>,
    pub totals: Vec<TsExpenseTotal>,
    pub refunded_totals: Vec<TsExpenseTotal>,
    pub protocol: Option<String>,
    pub sent_date: Option<String>,
    /// Last operation recorded on it: `I` inserimento, `V` variazione, `R` rimborso.
    pub send_kind: Option<String>,
    pub cancelled: bool,
    pub messages: Vec<TsMessage>,
}

/// Outcome of looking one document up on the Sistema TS.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TsQueryResult {
    Found { document: Box<TsRemoteDocument> },
    NotFound,
    Refused { messages: Vec<TsMessage> },
}
