use crate::app::model::ts::{TsDocumentId, TsExpenseDocument, TsReportBasis};

/// A call the fake received, with what identifies it.
#[derive(Debug, Clone, PartialEq)]
pub enum RecordedCall {
    Insert(TsExpenseDocument),
    Update(TsExpenseDocument),
    Cancel(TsDocumentId),
    Query(TsDocumentId),
    Report(i32, u32, TsReportBasis),
}
