use super::*;
use crate::test_support::email_database;

fn template(subject: &str, body: &str) -> EmailTemplate {
    EmailTemplate {
        subject: subject.to_string(),
        body: body.to_string(),
    }
}

#[tokio::test]
async fn starts_from_the_built_in_template_and_returns_to_it() {
    let db = email_database::setup(None).await;
    assert_eq!(get(&db).await.unwrap(), default_template());
    let saved = update(
        &db,
        template(" Fattura {numero_fattura} ", "Gentile {nome_paziente}\n"),
    )
    .await
    .unwrap();
    assert_eq!(
        saved,
        template("Fattura {numero_fattura}", "Gentile {nome_paziente}")
    );
    assert_eq!(get(&db).await.unwrap(), saved);
    assert_eq!(reset(&db).await.unwrap(), default_template());
    assert_eq!(get(&db).await.unwrap(), default_template());
}

#[tokio::test]
async fn refuses_unknown_placeholders_naming_them() {
    let db = email_database::setup(None).await;
    let error = update(
        &db,
        template("Fattura {numero}", "Gentile {paziente}, {saldo}"),
    )
    .await
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "Segnaposto sconosciuti: {numero}, {saldo}"
    );
}

#[tokio::test]
async fn refuses_empty_or_multiline_subjects_and_empty_bodies() {
    let db = email_database::setup(None).await;
    assert!(update(&db, template("", "testo")).await.is_err());
    assert!(update(&db, template("a\nb", "testo")).await.is_err());
    assert!(update(&db, template("oggetto", "  ")).await.is_err());
}

#[tokio::test]
async fn previews_with_an_example_patient_and_the_real_professional() {
    let db = email_database::setup(None).await;
    let preview = preview(&db, default_template()).await.unwrap();
    assert!(preview.subject.ends_with("– Dott.ssa Maria Demo"));
    assert!(preview.body.starts_with("Gentile Anna Bianchi,"));
    assert!(preview.body.contains("81,60 €"));
    assert!(!preview.body.contains('{'));
}

#[test]
fn every_placeholder_is_offered_and_the_default_uses_only_known_ones() {
    assert_eq!(placeholders().len(), EmailPlaceholder::ALL.len());
    assert!(email_template_renderer::unknown_placeholders(DEFAULT_BODY).is_empty());
    assert!(email_template_renderer::unknown_placeholders(DEFAULT_SUBJECT).is_empty());
}
