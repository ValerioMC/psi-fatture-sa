use crate::app::model::ts::{TsDocumentId, TsExpenseDocument};

/// The Sistema TS call a queued submission turns into.
pub(super) enum TsDispatchRequest {
    Insert(TsExpenseDocument),
    Update(TsExpenseDocument),
    Cancel(TsDocumentId),
}

impl TsDispatchRequest {
    pub(super) fn document_id(&self) -> &TsDocumentId {
        match self {
            TsDispatchRequest::Insert(document) | TsDispatchRequest::Update(document) => {
                &document.id
            }
            TsDispatchRequest::Cancel(id) => id,
        }
    }
}
