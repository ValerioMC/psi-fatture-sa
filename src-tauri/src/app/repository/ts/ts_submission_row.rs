use sea_orm::FromQueryResult;

use crate::app::common::AppError;
use crate::app::model::ts::{
    TsDocumentId, TsEnvironment, TsOperation, TsSubmission, TsSubmissionStatus,
};

/// A `ts_submissions` row joined with its invoice number and client name.
#[derive(FromQueryResult)]
pub(super) struct TsSubmissionRow {
    id: i64,
    invoice_id: i64,
    invoice_number: String,
    invoice_year: i64,
    client_name: String,
    operation: String,
    status: String,
    target_submission_id: Option<i64>,
    environment: String,
    document_vat_number: Option<String>,
    document_issue_date: Option<String>,
    document_number: Option<String>,
    protocol: Option<String>,
    outcome_code: Option<String>,
    outcome_message: Option<String>,
    attempt_count: i64,
    last_error: Option<String>,
    next_attempt_at: String,
    last_attempt_at: Option<String>,
    sent_at: Option<String>,
    resolved_at: Option<String>,
    created_at: String,
    updated_at: String,
}

impl TsSubmissionRow {
    pub(super) fn into_submission(self) -> Result<TsSubmission, AppError> {
        let document = match (
            self.document_vat_number,
            self.document_issue_date,
            self.document_number,
        ) {
            (Some(vat_number), Some(issue_date), Some(number)) => Some(TsDocumentId {
                vat_number,
                issue_date,
                number,
            }),
            _ => None,
        };
        Ok(TsSubmission {
            id: self.id,
            invoice_id: self.invoice_id,
            invoice_number: self.invoice_number,
            invoice_year: self.invoice_year,
            client_name: self.client_name,
            operation: TsOperation::parse(&self.operation)?,
            status: TsSubmissionStatus::parse(&self.status)?,
            target_submission_id: self.target_submission_id,
            environment: TsEnvironment::parse(&self.environment)?,
            document,
            protocol: self.protocol,
            outcome_code: self.outcome_code,
            outcome_message: self.outcome_message,
            attempt_count: self.attempt_count,
            last_error: self.last_error,
            next_attempt_at: self.next_attempt_at,
            last_attempt_at: self.last_attempt_at,
            sent_at: self.sent_at,
            resolved_at: self.resolved_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
