use super::*;

fn message(code: &str, description: &str) -> TsMessage {
    TsMessage {
        code: code.to_string(),
        description: description.to_string(),
        kind: "E".to_string(),
    }
}

#[test]
fn joins_code_and_description_of_each_message_with_a_dot() {
    let messages = [message("S017", "Codice errato"), message("W008", "Avviso")];
    assert_eq!(
        TsMessage::joined(&messages),
        "S017 Codice errato · W008 Avviso"
    );
}

#[test]
fn no_messages_join_to_an_empty_line() {
    assert_eq!(TsMessage::joined(&[]), "");
}
