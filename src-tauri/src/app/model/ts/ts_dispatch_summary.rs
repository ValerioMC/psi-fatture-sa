use serde::Serialize;

/// What one pass over the queue did. `blocked` says why the pass stopped,
/// such as missing or refused credentials.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct TsDispatchSummary {
    pub accepted: u32,
    pub rejected: u32,
    pub retrying: u32,
    /// Queued for the environment not currently selected, so left alone.
    pub waiting_other_environment: u32,
    pub blocked: Option<String>,
}
