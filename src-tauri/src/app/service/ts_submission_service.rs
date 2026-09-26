//! The user's side of the Sistema TS queue: queueing a first transmission,
//! a replacement or a cancellation of an accepted one, and withdrawing
//! whatever has not left the machine yet. Sending is the worker's job.

use sea_orm::{ActiveValue::Set, ConnectionTrait, DatabaseConnection, TransactionTrait};

use crate::app::entity::ts_submission::ActiveModel;
use crate::app::model::ts::{
    TsDocumentId, TsEnvironment, TsOperation, TsSubmission, TsSubmissionFilters, TsSubmissionStatus,
};
use crate::app::repository::ts_submission_repository;
use crate::app::service::{
    ts_document_service, ts_settings_service, validation_service as validate,
};

const IN_FLIGHT: [TsSubmissionStatus; 2] =
    [TsSubmissionStatus::NonInviata, TsSubmissionStatus::Inviata];

pub async fn list(
    db: &DatabaseConnection,
    filters: TsSubmissionFilters,
) -> Result<Vec<TsSubmission>, String> {
    ts_submission_repository::find(db, &filters).await
}

/// Queues the first transmission of a paid invoice to the configured
/// environment, checking now that the document can be built.
pub async fn enqueue_invio(
    db: &DatabaseConnection,
    invoice_id: i64,
) -> Result<TsSubmission, String> {
    validate::validate_id(invoice_id, "Fattura")?;
    let tx = db.begin().await.map_err(|e| e.to_string())?;

    let settings = ts_settings_service::get(&tx).await?;
    if settings.vat_number.trim().is_empty() {
        return Err("Indica la partita IVA del Sistema TS nelle Impostazioni".to_string());
    }
    ts_document_service::build(&tx, invoice_id, &settings.vat_number, None).await?;
    ensure_nothing_in_flight(&tx, invoice_id).await?;
    if ts_submission_repository::invoice_has_status(
        &tx,
        invoice_id,
        &[TsSubmissionStatus::Accettata],
        Some(settings.environment),
    )
    .await?
    {
        return Err(
            "Fattura già accettata dal Sistema TS: per correggerla usa la sostituzione".to_string(),
        );
    }

    let created = ts_submission_repository::insert(
        &tx,
        new_submission(invoice_id, TsOperation::Invio, None, settings.environment),
    )
    .await?;
    tx.commit().await.map_err(|e| e.to_string())?;
    ts_submission_repository::load(db, created.id).await
}

/// Queues a replacement of an accepted submission with the invoice's current data.
pub async fn enqueue_replacement(
    db: &DatabaseConnection,
    target_submission_id: i64,
) -> Result<TsSubmission, String> {
    enqueue_follow_up(db, target_submission_id, TsOperation::Sostituzione).await
}

/// Queues the cancellation of an accepted submission.
pub async fn enqueue_cancellation(
    db: &DatabaseConnection,
    target_submission_id: i64,
) -> Result<TsSubmission, String> {
    enqueue_follow_up(db, target_submission_id, TsOperation::Annullamento).await
}

/// Removes a submission that is still waiting in the queue.
pub async fn withdraw(db: &DatabaseConnection, submission_id: i64) -> Result<(), String> {
    validate::validate_id(submission_id, "Trasmissione STS")?;
    let row = ts_submission_repository::find_row(db, submission_id).await?;
    if TsSubmissionStatus::parse(&row.status)? != TsSubmissionStatus::NonInviata {
        return Err(
            "Solo le trasmissioni non ancora inviate possono essere ritirate dalla coda"
                .to_string(),
        );
    }
    ts_submission_repository::delete(db, submission_id).await
}

/// Whether deleting the invoice would orphan data the production Sistema TS holds.
pub async fn blocks_invoice_deletion(
    db: &DatabaseConnection,
    invoice_id: i64,
) -> Result<bool, String> {
    ts_submission_repository::invoice_holds_ts_data(db, invoice_id, TsEnvironment::Produzione).await
}

// ─── Private helpers ──────────────────────────────────────────────────────────

async fn enqueue_follow_up(
    db: &DatabaseConnection,
    target_submission_id: i64,
    operation: TsOperation,
) -> Result<TsSubmission, String> {
    validate::validate_id(target_submission_id, "Trasmissione STS")?;
    let tx = db.begin().await.map_err(|e| e.to_string())?;

    let target = ts_submission_repository::find_row(&tx, target_submission_id).await?;
    if TsSubmissionStatus::parse(&target.status)? != TsSubmissionStatus::Accettata {
        return Err(format!(
            "Si può chiedere {} solo di una trasmissione accettata",
            follow_up_label(operation)
        ));
    }
    let environment = TsEnvironment::parse(&target.environment)?;
    if operation == TsOperation::Sostituzione {
        let id = document_id_of(&target);
        let vat_number = id
            .as_ref()
            .map(|i| i.vat_number.clone())
            .unwrap_or_default();
        ts_document_service::build(&tx, target.invoice_id, &vat_number, id).await?;
    }
    ensure_nothing_in_flight(&tx, target.invoice_id).await?;

    let created = ts_submission_repository::insert(
        &tx,
        new_submission(target.invoice_id, operation, Some(target.id), environment),
    )
    .await?;
    tx.commit().await.map_err(|e| e.to_string())?;
    ts_submission_repository::load(db, created.id).await
}

/// The id an accepted submission was sent under, which follow-ups must reuse.
pub(crate) fn document_id_of(
    row: &crate::app::entity::ts_submission::Model,
) -> Option<TsDocumentId> {
    match (
        &row.document_vat_number,
        &row.document_issue_date,
        &row.document_number,
    ) {
        (Some(vat_number), Some(issue_date), Some(number)) => Some(TsDocumentId {
            vat_number: vat_number.clone(),
            issue_date: issue_date.clone(),
            number: number.clone(),
        }),
        _ => None,
    }
}

async fn ensure_nothing_in_flight(
    db: &impl ConnectionTrait,
    invoice_id: i64,
) -> Result<(), String> {
    if ts_submission_repository::invoice_has_status(db, invoice_id, &IN_FLIGHT, None).await? {
        return Err(
            "C'è già una trasmissione in corso per questa fattura: attendi l'esito".to_string(),
        );
    }
    Ok(())
}

fn new_submission(
    invoice_id: i64,
    operation: TsOperation,
    target_submission_id: Option<i64>,
    environment: TsEnvironment,
) -> ActiveModel {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    ActiveModel {
        invoice_id: Set(invoice_id),
        operation: Set(operation.as_str().to_owned()),
        status: Set(TsSubmissionStatus::NonInviata.as_str().to_owned()),
        target_submission_id: Set(target_submission_id),
        environment: Set(environment.as_str().to_owned()),
        attempt_count: Set(0),
        next_attempt_at: Set(now.clone()),
        created_at: Set(now.clone()),
        updated_at: Set(now),
        ..Default::default()
    }
}

fn follow_up_label(operation: TsOperation) -> &'static str {
    match operation {
        TsOperation::Sostituzione => "la sostituzione",
        _ => "l'annullamento",
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;

    pub(crate) async fn setup() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        crate::migration::Migrator::up(&db, None).await.unwrap();
        db.execute_unprepared(
            "INSERT INTO clients (id, first_name, last_name, fiscal_code, sts_authorization)
               VALUES (1, 'Anna', 'Bianchi', 'RSSMRA80A41H501Y', 1);
             INSERT INTO invoices (id, client_id, invoice_number, year, issue_date, status, paid_date, total_gross)
               VALUES (1, 1, '1', 2026, '2026-03-01', 'paid', '2026-03-02', 81.64),
                      (2, 1, '2', 2026, '2026-03-05', 'issued', NULL, 81.64),
                      (3, 1, '3', 2026, '2026-03-06', 'paid', NULL, 81.64);
             INSERT INTO ts_settings (id, environment, username, vat_number)
               VALUES (1, 'test', 'MTOMRA66A41G224M', '65498732105');",
        )
        .await
        .unwrap();
        db
    }

    pub(crate) async fn force_status(db: &DatabaseConnection, id: i64, status: TsSubmissionStatus) {
        db.execute_unprepared(&format!(
            "UPDATE ts_submissions SET status = '{}' WHERE id = {id}",
            status.as_str()
        ))
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn queues_a_paid_invoice_with_its_details() {
        let db = setup().await;
        let queued = enqueue_invio(&db, 1).await.unwrap();
        assert_eq!(queued.status, TsSubmissionStatus::NonInviata);
        assert_eq!(queued.operation, TsOperation::Invio);
        assert_eq!(queued.invoice_number, "1");
        assert_eq!(queued.client_name, "Anna Bianchi");
        assert_eq!(queued.attempt_count, 0);
    }

    #[tokio::test]
    async fn refuses_unpaid_or_undated_invoices() {
        let db = setup().await;
        assert!(enqueue_invio(&db, 2).await.unwrap_err().contains("pagate"));
        assert!(enqueue_invio(&db, 3)
            .await
            .unwrap_err()
            .contains("data di pagamento"));
        assert!(enqueue_invio(&db, 99).await.is_err());
    }

    #[tokio::test]
    async fn allows_one_in_flight_submission_per_invoice() {
        let db = setup().await;
        let first = enqueue_invio(&db, 1).await.unwrap();
        assert!(enqueue_invio(&db, 1)
            .await
            .unwrap_err()
            .contains("in corso"));

        force_status(&db, first.id, TsSubmissionStatus::Inviata).await;
        assert!(enqueue_invio(&db, 1)
            .await
            .unwrap_err()
            .contains("in corso"));
    }

    #[tokio::test]
    async fn requeues_after_rejection_but_not_after_acceptance() {
        let db = setup().await;
        let first = enqueue_invio(&db, 1).await.unwrap();
        force_status(&db, first.id, TsSubmissionStatus::Scartata).await;
        let retry = enqueue_invio(&db, 1).await.unwrap();

        force_status(&db, retry.id, TsSubmissionStatus::Accettata).await;
        assert!(enqueue_invio(&db, 1)
            .await
            .unwrap_err()
            .contains("sostituzione"));
    }

    #[tokio::test]
    async fn follow_ups_target_only_accepted_submissions() {
        let db = setup().await;
        let original = enqueue_invio(&db, 1).await.unwrap();
        assert!(enqueue_cancellation(&db, original.id).await.is_err());

        force_status(&db, original.id, TsSubmissionStatus::Accettata).await;
        let cancellation = enqueue_cancellation(&db, original.id).await.unwrap();
        assert_eq!(cancellation.operation, TsOperation::Annullamento);
        assert_eq!(cancellation.target_submission_id, Some(original.id));

        assert!(enqueue_replacement(&db, original.id)
            .await
            .unwrap_err()
            .contains("in corso"));
    }

    #[tokio::test]
    async fn replacement_requires_the_invoice_to_still_be_paid() {
        let db = setup().await;
        let original = enqueue_invio(&db, 1).await.unwrap();
        force_status(&db, original.id, TsSubmissionStatus::Accettata).await;
        db.execute_unprepared("UPDATE invoices SET status = 'issued' WHERE id = 1")
            .await
            .unwrap();

        assert!(enqueue_replacement(&db, original.id)
            .await
            .unwrap_err()
            .contains("pagate"));
        assert!(enqueue_cancellation(&db, original.id).await.is_ok());
    }

    #[tokio::test]
    async fn withdraws_only_unsent_submissions() {
        let db = setup().await;
        let queued = enqueue_invio(&db, 1).await.unwrap();
        withdraw(&db, queued.id).await.unwrap();
        assert!(list(&db, TsSubmissionFilters::default())
            .await
            .unwrap()
            .is_empty());

        let sent = enqueue_invio(&db, 1).await.unwrap();
        force_status(&db, sent.id, TsSubmissionStatus::Inviata).await;
        assert!(withdraw(&db, sent.id)
            .await
            .unwrap_err()
            .contains("non ancora inviate"));
    }

    #[tokio::test]
    async fn lists_by_invoice_and_status() {
        let db = setup().await;
        db.execute_unprepared(
            "UPDATE invoices SET status = 'paid', paid_date = '2026-03-07' WHERE id = 3",
        )
        .await
        .unwrap();
        let first = enqueue_invio(&db, 1).await.unwrap();
        enqueue_invio(&db, 3).await.unwrap();
        force_status(&db, first.id, TsSubmissionStatus::Accettata).await;

        let for_invoice = list(
            &db,
            TsSubmissionFilters {
                invoice_id: Some(3),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(for_invoice.len(), 1);
        assert_eq!(for_invoice[0].invoice_id, 3);

        let accepted = list(
            &db,
            TsSubmissionFilters {
                status: Some(TsSubmissionStatus::Accettata),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].id, first.id);
    }

    #[tokio::test]
    async fn blocks_deletion_while_production_holds_the_data() {
        let db = setup().await;
        use_production(&db).await;
        let queued = enqueue_invio(&db, 1).await.unwrap();
        assert!(!blocks_invoice_deletion(&db, 1).await.unwrap());

        force_status(&db, queued.id, TsSubmissionStatus::Accettata).await;
        assert!(blocks_invoice_deletion(&db, 1).await.unwrap());

        force_status(&db, queued.id, TsSubmissionStatus::Annullata).await;
        assert!(!blocks_invoice_deletion(&db, 1).await.unwrap());
    }

    #[tokio::test]
    async fn test_environment_data_never_blocks_deletion() {
        let db = setup().await;
        let queued = enqueue_invio(&db, 1).await.unwrap();
        force_status(&db, queued.id, TsSubmissionStatus::Accettata).await;
        assert!(!blocks_invoice_deletion(&db, 1).await.unwrap());
    }

    async fn use_production(db: &DatabaseConnection) {
        db.execute_unprepared("UPDATE ts_settings SET environment = 'produzione'")
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn invoice_deletion_is_refused_while_accepted_and_cascades_after_cancellation() {
        use crate::app::service::invoice_service;

        let db = setup().await;
        use_production(&db).await;
        let original = enqueue_invio(&db, 1).await.unwrap();
        force_status(&db, original.id, TsSubmissionStatus::Accettata).await;
        let err = invoice_service::remove(&db, 1).await.unwrap_err();
        assert!(err.contains("Sistema TS"));

        let cancellation = enqueue_cancellation(&db, original.id).await.unwrap();
        force_status(&db, cancellation.id, TsSubmissionStatus::Accettata).await;
        force_status(&db, original.id, TsSubmissionStatus::Annullata).await;
        db.execute_unprepared("PRAGMA foreign_keys = ON")
            .await
            .unwrap();

        invoice_service::remove(&db, 1).await.unwrap();
        let left = list(&db, TsSubmissionFilters::default()).await.unwrap();
        assert!(left.is_empty());
    }
}
