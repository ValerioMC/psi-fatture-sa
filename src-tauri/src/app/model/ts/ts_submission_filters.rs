use serde::Deserialize;

use super::TsSubmissionStatus;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TsSubmissionFilters {
    pub invoice_id: Option<i64>,
    pub year: Option<i64>,
    pub status: Option<TsSubmissionStatus>,
}
