use serde::{Deserialize, Serialize};

use super::ts_document::TsDocumentId;
use super::ts_environment::TsEnvironment;
use super::ts_operation::TsOperation;
use super::ts_submission_status::TsSubmissionStatus;

/// One queued or completed Sistema TS transmission, with the invoice it reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsSubmission {
    pub id: i64,
    pub invoice_id: i64,
    pub invoice_number: String,
    pub invoice_year: i64,
    pub client_name: String,
    pub operation: TsOperation,
    pub status: TsSubmissionStatus,
    pub target_submission_id: Option<i64>,
    pub environment: TsEnvironment,
    /// The id the document was sent under, known once the first attempt starts.
    pub document: Option<TsDocumentId>,
    pub protocol: Option<String>,
    pub outcome_code: Option<String>,
    pub outcome_message: Option<String>,
    pub attempt_count: i64,
    pub last_error: Option<String>,
    pub next_attempt_at: String,
    pub last_attempt_at: Option<String>,
    pub sent_at: Option<String>,
    pub resolved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
