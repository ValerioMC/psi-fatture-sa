use sea_orm::{ConnectionTrait, Database};
use sea_orm_migration::MigratorTrait;

use super::*;
use crate::test_support::clock::at;

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("psi-rotation-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

async fn live_database(dir: &Path) -> DatabaseConnection {
    let db = Database::connect(format!(
        "sqlite://{}?mode=rwc",
        dir.join("live.db").display()
    ))
    .await
    .unwrap();
    crate::migration::Migrator::up(&db, None).await.unwrap();
    db
}

fn rotation(dir: &Path) -> RotatingBackups {
    RotatingBackups::new(dir.join("live.db"), dir.join("backups"))
}

#[tokio::test]
async fn a_backup_is_a_database_with_the_same_rows() {
    let dir = scratch_dir("rows");
    let db = live_database(&dir).await;
    db.execute_unprepared(
        "INSERT INTO clients (first_name, last_name, fiscal_code) VALUES ('Anna', 'Bianchi', 'X')",
    )
    .await
    .unwrap();

    let file = rotation(&dir)
        .back_up(&db, BackupReason::Manual, at("2026-10-03 09:15:00"))
        .await
        .unwrap();

    assert_eq!(file.name, "psi-fatture-20261003-091500-manual.db");
    assert_eq!(file.created_at, "2026-10-03 09:15:00");
    let copy = Database::connect(format!("sqlite://{}?mode=ro", file.path))
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
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn only_the_newest_ten_are_kept() {
    let dir = scratch_dir("prune");
    let db = live_database(&dir).await;
    let backups = rotation(&dir);

    for day in 1..=12 {
        backups
            .back_up(
                &db,
                BackupReason::Auto,
                at(&format!("2026-09-{day:02} 10:00:00")),
            )
            .await
            .unwrap();
    }

    let kept = backups.list().unwrap();
    assert_eq!(kept.len(), KEEP);
    assert_eq!(kept.first().unwrap().created_at, "2026-09-12 10:00:00");
    assert_eq!(kept.last().unwrap().created_at, "2026-09-03 10:00:00");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn two_backups_in_the_same_second_both_survive_newest_first() {
    let dir = scratch_dir("same-second");
    let db = live_database(&dir).await;
    let backups = rotation(&dir);
    let now = at("2026-10-03 09:15:00");

    backups.back_up(&db, BackupReason::Auto, now).await.unwrap();
    let second = backups
        .back_up(&db, BackupReason::Manual, now)
        .await
        .unwrap();

    assert_eq!(second.name, "psi-fatture-20261003-091500-1-manual.db");
    let names: Vec<String> = backups
        .list()
        .unwrap()
        .into_iter()
        .map(|f| f.name)
        .collect();
    assert_eq!(
        names,
        vec![
            "psi-fatture-20261003-091500-1-manual.db",
            "psi-fatture-20261003-091500-auto.db"
        ]
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn the_daily_backup_is_skipped_once_today_has_one() {
    let dir = scratch_dir("due");
    let db = live_database(&dir).await;
    let backups = rotation(&dir);

    let first = backups
        .back_up_if_due(&db, at("2026-10-03 08:00:00"))
        .await
        .unwrap();
    let again = backups
        .back_up_if_due(&db, at("2026-10-03 21:00:00"))
        .await
        .unwrap();
    let tomorrow = backups
        .back_up_if_due(&db, at("2026-10-04 08:00:00"))
        .await
        .unwrap();

    assert_eq!(first.unwrap().reason, BackupReason::Auto);
    assert!(again.is_none());
    assert!(tomorrow.is_some());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn a_failed_copy_is_reported_until_one_succeeds() {
    let dir = scratch_dir("failure");
    let db = live_database(&dir).await;
    std::fs::write(dir.join("backups"), b"a file where the folder should be").unwrap();
    let backups = rotation(&dir);

    let failed = backups
        .back_up(&db, BackupReason::Auto, at("2026-10-03 08:00:00"))
        .await;

    assert!(matches!(failed, Err(AppError::External(_))));
    assert!(backups.overview().last_failure.is_some());
    std::fs::remove_file(dir.join("backups")).unwrap();
    backups
        .back_up(&db, BackupReason::Auto, at("2026-10-03 08:01:00"))
        .await
        .unwrap();
    assert_eq!(backups.overview().last_failure, None);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn foreign_files_in_the_folder_are_not_backups() {
    let dir = scratch_dir("foreign");
    let folder = dir.join("backups");
    std::fs::create_dir_all(&folder).unwrap();
    for name in [
        "notes.txt",
        ".psi-fatture-20261003-091500-auto.db.partial",
        "psi-fatture-20261003-091500-weekly.db",
        "psi-fatture-2026103-091500-auto.db",
        "psi-fatture-20261003-091500-auto.db",
    ] {
        std::fs::write(folder.join(name), b"x").unwrap();
    }

    let names: Vec<String> = rotation(&dir)
        .list()
        .unwrap()
        .into_iter()
        .map(|f| f.name)
        .collect();

    assert_eq!(names, vec!["psi-fatture-20261003-091500-auto.db"]);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_overview_names_the_database_and_the_folder() {
    let dir = scratch_dir("overview");

    let overview = rotation(&dir).overview();

    assert_eq!(
        overview.database_path,
        dir.join("live.db").to_string_lossy()
    );
    assert_eq!(
        overview.backups_folder,
        dir.join("backups").to_string_lossy()
    );
    assert_eq!(overview.keep, KEEP);
    assert!(overview.backups.is_empty());
    std::fs::remove_dir_all(&dir).unwrap();
}

async fn row_count(path: &Path, table: &str) -> i64 {
    let db = Database::connect(format!("sqlite://{}?mode=ro", path.display()))
        .await
        .unwrap();
    db.query_one(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        format!("SELECT COUNT(*) AS total FROM {table}"),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get::<i64>("", "total")
    .unwrap()
}

#[tokio::test]
async fn staging_a_restore_copies_the_chosen_backup_and_keeps_a_safety_copy() {
    let dir = scratch_dir("stage");
    let db = live_database(&dir).await;
    let backups = rotation(&dir);
    let chosen = backups
        .back_up(&db, BackupReason::Auto, at("2026-10-01 09:00:00"))
        .await
        .unwrap();
    db.execute_unprepared(
        "INSERT INTO clients (first_name, last_name, fiscal_code) VALUES ('Anna', 'Bianchi', 'X')",
    )
    .await
    .unwrap();

    backups
        .stage_restore(&db, &chosen.name, at("2026-10-05 18:00:00"))
        .await
        .unwrap();

    let staged = staged_restore_path(&dir.join("live.db"));
    assert_eq!(row_count(&staged, "clients").await, 0);
    let safety = &backups.list().unwrap()[0];
    assert_eq!(safety.created_at, "2026-10-05 18:00:00");
    assert_eq!(row_count(Path::new(&safety.path), "clients").await, 1);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn a_backup_that_is_not_in_the_rotation_is_not_found() {
    let dir = scratch_dir("unknown");
    let db = live_database(&dir).await;

    let error = rotation(&dir)
        .stage_restore(
            &db,
            "psi-fatture-20250101-000000-auto.db",
            at("2026-10-05 18:00:00"),
        )
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::NotFound(_)));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn a_file_that_is_not_an_archive_is_refused_and_nothing_is_staged() {
    let dir = scratch_dir("garbage");
    let db = live_database(&dir).await;
    let folder = dir.join("backups");
    std::fs::create_dir_all(&folder).unwrap();
    let name = "psi-fatture-20261001-090000-auto.db";
    std::fs::write(folder.join(name), b"not a database at all").unwrap();

    let error = rotation(&dir)
        .stage_restore(&db, name, at("2026-10-05 18:00:00"))
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::Invalid(_)));
    assert!(!staged_restore_path(&dir.join("live.db")).exists());
    assert_eq!(rotation(&dir).list().unwrap().len(), 1);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn a_backup_from_a_newer_app_is_refused() {
    let dir = scratch_dir("newer");
    let db = live_database(&dir).await;
    let backups = rotation(&dir);
    let chosen = backups
        .back_up(&db, BackupReason::Auto, at("2026-10-01 09:00:00"))
        .await
        .unwrap();
    let copy = Database::connect(format!("sqlite://{}?mode=rw", chosen.path))
        .await
        .unwrap();
    copy.execute_unprepared(
        "INSERT INTO seaql_migrations (version, applied_at) VALUES ('m29990101_000000_from_the_future', 0)",
    )
    .await
    .unwrap();
    copy.close().await.unwrap();

    let error = backups
        .stage_restore(&db, &chosen.name, at("2026-10-05 18:00:00"))
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::Conflict(_)));
    assert!(!staged_restore_path(&dir.join("live.db")).exists());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_staged_restore_replaces_the_database_and_its_journal_files() {
    let dir = scratch_dir("apply");
    let database = dir.join("live.db");
    std::fs::write(&database, b"old data").unwrap();
    std::fs::write(dir.join("live.db-wal"), b"old journal").unwrap();
    std::fs::write(dir.join("live.db-shm"), b"old index").unwrap();
    std::fs::write(staged_restore_path(&database), b"restored data").unwrap();

    apply_staged_restore(&database).unwrap();

    assert_eq!(std::fs::read(&database).unwrap(), b"restored data");
    assert!(!dir.join("live.db-wal").exists());
    assert!(!dir.join("live.db-shm").exists());
    assert!(!staged_restore_path(&database).exists());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn without_a_staged_restore_the_database_is_left_alone() {
    let dir = scratch_dir("untouched");
    let database = dir.join("live.db");
    std::fs::write(&database, b"current data").unwrap();
    std::fs::write(dir.join("live.db-wal"), b"journal").unwrap();

    apply_staged_restore(&database).unwrap();

    assert_eq!(std::fs::read(&database).unwrap(), b"current data");
    assert!(dir.join("live.db-wal").exists());
    std::fs::remove_dir_all(&dir).unwrap();
}
