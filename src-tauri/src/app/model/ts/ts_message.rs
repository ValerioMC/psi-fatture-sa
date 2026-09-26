use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TsMessage {
    pub code: String,
    pub description: String,
    /// `E` blocking error, `W` warning, `S` statistic, empty for the success line.
    pub kind: String,
}

impl TsMessage {
    pub fn is_error(&self) -> bool {
        self.kind == "E"
    }
}
