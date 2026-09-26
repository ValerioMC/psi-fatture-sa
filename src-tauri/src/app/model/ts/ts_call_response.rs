use super::{TsEsito, TsMessage};

/// The call itself did not go through: retry as is.
const TRANSIENT_CODES: [&str; 2] = ["WS99", "200"];
/// The credentials or the owner were refused: retry once they are fixed.
const CREDENTIAL_CODES: [&str; 9] = [
    "002", "003", "004", "005", "006", "010", "104", "107", "110",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TsCallResponse {
    pub esito: TsEsito,
    pub protocol: Option<String>,
    pub messages: Vec<TsMessage>,
}

impl TsCallResponse {
    pub fn accepted(&self) -> bool {
        self.esito != TsEsito::Rejected
    }

    pub fn has_code(&self, code: &str) -> bool {
        self.messages.iter().any(|m| m.code == code)
    }

    /// A rejection caused by the call or the credentials, not by the document.
    pub fn is_retryable_rejection(&self) -> bool {
        self.rejected_with(&TRANSIENT_CODES) || self.is_credential_rejection()
    }

    pub fn is_credential_rejection(&self) -> bool {
        self.rejected_with(&CREDENTIAL_CODES)
    }

    fn rejected_with(&self, codes: &[&str]) -> bool {
        !self.accepted()
            && self
                .messages
                .iter()
                .any(|m| m.is_error() && codes.contains(&m.code.as_str()))
    }

    /// The error codes, or every code when there is no error, joined for storage.
    pub fn summary_code(&self) -> Option<String> {
        let errors: Vec<&str> = self
            .messages
            .iter()
            .filter(|m| m.is_error())
            .map(|m| m.code.as_str())
            .collect();
        let codes = if errors.is_empty() {
            self.messages
                .iter()
                .filter(|m| m.kind == "W")
                .map(|m| m.code.as_str())
                .collect()
        } else {
            errors
        };
        (!codes.is_empty()).then(|| codes.join(", "))
    }

    /// Every error and warning description, skipping the plain success line.
    pub fn summary_message(&self) -> Option<String> {
        let lines: Vec<String> = self
            .messages
            .iter()
            .filter(|m| m.is_error() || m.kind == "W")
            .map(|m| format!("{} {}", m.code, m.description))
            .collect();
        (!lines.is_empty()).then(|| lines.join(" · "))
    }
}

#[cfg(test)]
#[path = "ts_call_response_test.rs"]
mod tests;
