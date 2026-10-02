use sea_orm::{ConnectionTrait, Database};
use sea_orm_migration::MigratorTrait;

use super::*;

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("psi-backup-test-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[tokio::test]
async fn the_backup_is_a_database_with_the_same_rows() {
    let dir = scratch_dir("rows");
    let live = dir.join("live.db");
    let _ = std::fs::remove_file(&live);
    let db = Database::connect(format!("sqlite://{}?mode=rwc", live.display()))
        .await
        .unwrap();
    crate::migration::Migrator::up(&db, None).await.unwrap();
    db.execute_unprepared(
        "INSERT INTO clients (first_name, last_name, fiscal_code) VALUES ('Anna', 'Bianchi', 'X')",
    )
    .await
    .unwrap();
    let target = dir.join("backup.db");
    std::fs::write(&target, b"older backup").unwrap();

    export(&db, &target).await.unwrap();

    let copy = Database::connect(format!("sqlite://{}?mode=ro", target.display()))
        .await
        .unwrap();
    let row = copy
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT last_name FROM clients",
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.try_get::<String>("", "last_name").unwrap(), "Bianchi");
    assert!(!staging_path(&target).exists());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn an_empty_target_is_refused() {
    let db = Database::connect("sqlite::memory:").await.unwrap();

    let error = export(&db, Path::new("")).await.unwrap_err();

    assert!(matches!(error, AppError::Invalid(_)));
}

#[test]
fn the_proposed_name_carries_the_date() {
    let today = chrono::NaiveDate::from_ymd_opt(2026, 10, 2).unwrap();

    assert_eq!(file_name(today), "PSI-Fatture-backup-2026-10-02.db");
}
