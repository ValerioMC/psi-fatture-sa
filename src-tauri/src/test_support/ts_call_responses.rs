//! Canned Sistema TS answers for the fake gateway and the tests that script it.

use crate::app::model::ts::{TsCallResponse, TsEsito, TsMessage};

pub fn accepted(protocol: &str) -> TsCallResponse {
    TsCallResponse {
        esito: TsEsito::Accepted,
        protocol: Some(protocol.to_string()),
        messages: vec![message("0", "")],
    }
}

pub fn rejected(code: &str) -> TsCallResponse {
    TsCallResponse {
        esito: TsEsito::Rejected,
        protocol: None,
        messages: vec![message(code, "E")],
    }
}

pub fn message(code: &str, kind: &str) -> TsMessage {
    TsMessage {
        code: code.to_string(),
        description: format!("descrizione {code}"),
        kind: kind.to_string(),
    }
}
