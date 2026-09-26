use serde::Serialize;

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
