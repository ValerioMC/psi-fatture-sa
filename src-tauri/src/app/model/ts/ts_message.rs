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

    /// The messages as one line for the user: `code description`, separated by ` · `.
    pub fn joined<'a>(messages: impl IntoIterator<Item = &'a TsMessage>) -> String {
        messages
            .into_iter()
            .map(|message| format!("{} {}", message.code, message.description))
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

#[cfg(test)]
#[path = "ts_message_test.rs"]
mod tests;
