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
