use serde::Serialize;

/// The call itself did not go through: retry as is.
const TRANSIENT_CODES: [&str; 2] = ["WS99", "200"];
/// The credentials or the owner were refused: retry once they are fixed.
const CREDENTIAL_CODES: [&str; 9] = [
    "002", "003", "004", "005", "006", "010", "104", "107", "110",
];

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

/// `esitoChiamata` of a document call: 0 accepted, 2 accepted with warnings, 1 rejected.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TsEsito {
    Accepted,
    AcceptedWithWarnings,
    Rejected,
}

impl TsEsito {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim() {
            "0" => Ok(TsEsito::Accepted),
            "2" => Ok(TsEsito::AcceptedWithWarnings),
            "1" => Ok(TsEsito::Rejected),
            other => Err(format!("esitoChiamata sconosciuto: {other}")),
        }
    }
}

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
mod tests {
    use super::*;

    fn message(code: &str, kind: &str) -> TsMessage {
        TsMessage {
            code: code.to_string(),
            description: format!("descrizione {code}"),
            kind: kind.to_string(),
        }
    }

    fn response(esito: TsEsito, messages: Vec<TsMessage>) -> TsCallResponse {
        TsCallResponse {
            esito,
            protocol: None,
            messages,
        }
    }

    #[test]
    fn parses_esito_codes() {
        assert_eq!(TsEsito::parse("0"), Ok(TsEsito::Accepted));
        assert_eq!(TsEsito::parse("2"), Ok(TsEsito::AcceptedWithWarnings));
        assert_eq!(TsEsito::parse(" 1 "), Ok(TsEsito::Rejected));
        assert!(TsEsito::parse("9").is_err());
    }

    #[test]
    fn transient_and_credential_errors_are_retryable() {
        for code in ["WS99", "005", "110"] {
            assert!(response(TsEsito::Rejected, vec![message(code, "E")]).is_retryable_rejection());
        }
        assert!(!response(TsEsito::Rejected, vec![message("S017", "E")]).is_retryable_rejection());
        assert!(response(TsEsito::Rejected, vec![message("005", "E")]).is_credential_rejection());
        assert!(!response(TsEsito::Rejected, vec![message("WS99", "E")]).is_credential_rejection());
        assert!(!response(TsEsito::Accepted, vec![message("0", "")]).is_retryable_rejection());
    }

    #[test]
    fn summaries_keep_errors_and_warnings_and_drop_the_success_line() {
        let warned = response(
            TsEsito::AcceptedWithWarnings,
            vec![message("W008", "W"), message("0", "")],
        );
        assert_eq!(warned.summary_code().as_deref(), Some("W008"));
        assert_eq!(
            warned.summary_message().as_deref(),
            Some("W008 descrizione W008")
        );

        let rejected = response(
            TsEsito::Rejected,
            vec![message("W008", "W"), message("S057", "E")],
        );
        assert_eq!(rejected.summary_code().as_deref(), Some("S057"));

        let clean = response(TsEsito::Accepted, vec![message("0", "")]);
        assert_eq!(clean.summary_code(), None);
        assert_eq!(clean.summary_message(), None);
    }
}
