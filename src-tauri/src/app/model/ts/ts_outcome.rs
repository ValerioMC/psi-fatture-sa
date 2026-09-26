/// The Sistema TS verdict on one transmission, as read from its response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TsOutcome {
    pub accepted: bool,
    pub protocol: Option<String>,
    pub code: Option<String>,
    pub message: Option<String>,
}
