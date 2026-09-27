use super::*;

fn sample(placeholder: EmailPlaceholder) -> String {
    match placeholder {
        EmailPlaceholder::Patient => "Anna Bianchi".into(),
        EmailPlaceholder::InvoiceNumber => "12/2026".into(),
        other => format!("<{}>", other.key()),
    }
}

#[test]
fn fills_known_placeholders_and_tolerates_spaces_inside_braces() {
    assert_eq!(
        render("Gentile {paziente}, fattura { numero_fattura }.", sample),
        "Gentile Anna Bianchi, fattura 12/2026."
    );
}

#[test]
fn leaves_unknown_names_and_stray_braces_as_written() {
    assert_eq!(render("{ignoto} {paziente", sample), "{ignoto} {paziente");
    assert_eq!(
        render("a { b } {paziente}}", sample),
        "a { b } Anna Bianchi}"
    );
    assert_eq!(render("{{paziente}}", sample), "{Anna Bianchi}");
}

#[test]
fn lists_unknown_names_once() {
    assert_eq!(
        unknown_placeholders("{paziente} {nome} {importo} {nome} {Paziente}"),
        vec!["nome".to_string(), "Paziente".to_string()]
    );
    assert!(unknown_placeholders("nessun segnaposto { aperto").is_empty());
}
