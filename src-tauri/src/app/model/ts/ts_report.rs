use serde::{Deserialize, Serialize};

/// Which date the monthly report selects documents by (`tipoEstrazione`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TsReportBasis {
    Invio,
    Pagamento,
}

impl TsReportBasis {
    pub fn code(&self) -> &'static str {
        match self {
            TsReportBasis::Invio => "I",
            TsReportBasis::Pagamento => "P",
        }
    }
}

/// One document of the Sistema TS monthly report, matched to a local invoice when possible.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TsReportRow {
    pub vat_number: String,
    pub issue_date: String,
    pub document_number: String,
    pub payment_date: String,
    pub protocol: String,
    pub sent_date: String,
    pub send_kind: String,
    pub amount: f64,
    pub refunded_amount: f64,
    pub invoice_id: Option<i64>,
}
