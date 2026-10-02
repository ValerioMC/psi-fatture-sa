use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

use super::*;

async fn database() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    crate::migration::Migrator::up(&db, None).await.unwrap();
    db
}

fn input(version: &str, terms_accepted: bool, clauses_approved: bool) -> AcceptTermsInput {
    AcceptTermsInput {
        version: version.to_string(),
        terms_accepted,
        clauses_approved,
    }
}

#[tokio::test]
async fn a_version_is_pending_until_accepted() {
    let db = database().await;
    assert_eq!(find(&db, "2026-10-01").await.unwrap(), None);

    let accepted = accept(&db, input(" 2026-10-01 ", true, true))
        .await
        .unwrap();
    assert_eq!(accepted.version, "2026-10-01");
    assert!(accepted.clauses_approved);
    assert_eq!(find(&db, "2026-10-01").await.unwrap(), Some(accepted));
}

#[tokio::test]
async fn a_new_version_needs_its_own_acceptance() {
    let db = database().await;
    accept(&db, input("2026-10-01", true, true)).await.unwrap();
    assert_eq!(find(&db, "2027-01-15").await.unwrap(), None);
}

#[tokio::test]
async fn accepting_twice_keeps_the_first_record() {
    let db = database().await;
    let first = accept(&db, input("2026-10-01", true, true)).await.unwrap();
    let second = accept(&db, input("2026-10-01", true, true)).await.unwrap();
    assert_eq!(first, second);
}

#[tokio::test]
async fn refuses_without_both_boxes_and_records_nothing() {
    let db = database().await;
    let terms = accept(&db, input("2026-10-01", false, true))
        .await
        .unwrap_err();
    assert_eq!(
        terms.to_string(),
        "Per usare l'app accetta le condizioni d'uso"
    );
    let clauses = accept(&db, input("2026-10-01", true, false))
        .await
        .unwrap_err();
    assert!(clauses.to_string().contains("1341"));
    assert_eq!(find(&db, "2026-10-01").await.unwrap(), None);
}

#[tokio::test]
async fn refuses_an_empty_or_overlong_version() {
    let db = database().await;
    assert!(accept(&db, input("  ", true, true)).await.is_err());
    assert!(find(&db, &"x".repeat(41)).await.is_err());
}
