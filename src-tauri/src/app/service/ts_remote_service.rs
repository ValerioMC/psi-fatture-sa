//! Reads what the Sistema TS actually holds: one invoice through the point
//! query, a month through the report, and a harmless call to check the
//! credentials. Nothing here changes the local queue.

use sea_orm::{ConnectionTrait, DatabaseConnection};

use crate::app::model::ts::{
    TsConnectionCheck, TsDocumentId, TsOperation, TsQueryResult, TsReportBasis, TsReportRow,
    TsSettings, TsSubmissionFilters, TsSubmissionStatus,
};
use crate::app::repository::secret_store::SecretStore;
use crate::app::repository::sistema_ts::gateway::{SistemaTsGateway, TsGatewayError};
use crate::app::repository::sistema_ts::response::ReportOutcome;
use crate::app::repository::{invoice_repository, ts_submission_repository};
use crate::app::service::{
    ts_credential_service, ts_settings_service, validation_service as validate,
};

/// Number used by the connection check; no real invoice carries it.
const PROBE_DOCUMENT_NUMBER: &str = "PSIFATTURE-CHECK";

/// Looks the invoice up on the Sistema TS, under the id it was last
/// accepted with, or its own id if it was never sent.
pub async fn query_invoice(
    db: &DatabaseConnection,
    store: &dyn SecretStore,
    gateway: &dyn SistemaTsGateway,
    invoice_id: i64,
) -> Result<TsQueryResult, String> {
    validate::validate_id(invoice_id, "Fattura")?;
    let settings = ts_settings_service::get(db).await?;
    let session = ts_credential_service::session(store, &settings)?;
    let id = match last_sent_id(db, invoice_id, &settings).await? {
        Some(id) => id,
        None => own_id(db, invoice_id, &settings).await?,
    };
    gateway
        .query(&session, &id)
        .await
        .map_err(|e| e.to_string())
}

/// The documents the Sistema TS holds for a month, by send or payment date,
/// each matched to the local invoice it came from when there is one.
pub async fn monthly_report(
    db: &DatabaseConnection,
    store: &dyn SecretStore,
    gateway: &dyn SistemaTsGateway,
    year: i32,
    month: u32,
    basis: TsReportBasis,
) -> Result<Vec<TsReportRow>, String> {
    validate::validate_year(i64::from(year))?;
    validate::validate_month(i64::from(month))?;
    let settings = ts_settings_service::get(db).await?;
    let session = ts_credential_service::session(store, &settings)?;

    let outcome = gateway
        .monthly_report(&session, year, month, basis)
        .await
        .map_err(|e| e.to_string())?;
    let mut rows = match outcome {
        ReportOutcome::Rows(rows) => rows,
        ReportOutcome::Refused(messages) => {
            let reasons: Vec<String> = messages
                .iter()
                .filter(|m| m.is_error())
                .map(|m| format!("{} {}", m.code, m.description))
                .collect();
            return Err(format!("Report non disponibile: {}", reasons.join(" · ")));
        }
    };
    for row in &mut rows {
        row.invoice_id =
            invoice_repository::find_id_by_issue(db, &row.issue_date, &row.document_number).await?;
    }
    Ok(rows)
}

/// Checks username, password and PINCODE with a query for a document that
/// does not exist: "not found" means every credential was accepted.
pub async fn check_connection(
    db: &DatabaseConnection,
    store: &dyn SecretStore,
    gateway: &dyn SistemaTsGateway,
) -> Result<TsConnectionCheck, String> {
    let settings = ts_settings_service::get(db).await?;
    let session = match ts_credential_service::session(store, &settings) {
        Ok(session) => session,
        Err(reason) => return Ok(failed(reason)),
    };
    let probe = TsDocumentId {
        vat_number: settings.vat_number.clone(),
        issue_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        number: PROBE_DOCUMENT_NUMBER.to_string(),
    };
    Ok(match gateway.query(&session, &probe).await {
        Ok(TsQueryResult::NotFound | TsQueryResult::Found { .. }) => TsConnectionCheck {
            ok: true,
            message: format!(
                "Credenziali accettate dal Sistema TS ({})",
                settings.environment.as_str()
            ),
        },
        Ok(TsQueryResult::Refused { messages }) => failed(
            messages
                .iter()
                .map(|m| format!("{} {}", m.code, m.description))
                .collect::<Vec<_>>()
                .join(" · "),
        ),
        Err(TsGatewayError::Authentication) => {
            failed("Utente o password del Sistema TS non validi".to_string())
        }
        Err(TsGatewayError::Network(detail)) => failed(format!(
            "Nessuna risposta dal Sistema TS ({detail}). Controlla la connessione: con una password errata il servizio a volte non risponde affatto"
        )),
        Err(error) => failed(error.to_string()),
    })
}

async fn last_sent_id(
    db: &impl ConnectionTrait,
    invoice_id: i64,
    settings: &TsSettings,
) -> Result<Option<TsDocumentId>, String> {
    let filters = TsSubmissionFilters {
        invoice_id: Some(invoice_id),
        ..Default::default()
    };
    Ok(ts_submission_repository::find(db, &filters)
        .await?
        .into_iter()
        .filter(|s| s.environment == settings.environment)
        .find(|s| {
            s.operation != TsOperation::Annullamento
                && matches!(
                    s.status,
                    TsSubmissionStatus::Accettata
                        | TsSubmissionStatus::Sostituita
                        | TsSubmissionStatus::Annullata
                )
        })
        .and_then(|s| s.document))
}

async fn own_id(
    db: &impl ConnectionTrait,
    invoice_id: i64,
    settings: &TsSettings,
) -> Result<TsDocumentId, String> {
    let invoice = invoice_repository::load_invoice(db, invoice_id).await?;
    Ok(TsDocumentId {
        vat_number: settings.vat_number.clone(),
        issue_date: invoice.issue_date,
        number: invoice.invoice_number,
    })
}

fn failed(message: String) -> TsConnectionCheck {
    TsConnectionCheck { ok: false, message }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::model::ts::{TsMessage, TsRemoteDocument};
    use crate::app::repository::secret_store::testing::InMemorySecretStore;
    use crate::app::repository::secret_store::SecretKind;
    use crate::app::repository::sistema_ts::gateway::testing::{FakeGateway, RecordedCall};
    use crate::app::service::ts_dispatch_service;
    use crate::app::service::ts_submission_service::{self as queue, tests::setup};
    use crate::app::service::ts_transition_service::tests::at;

    fn store() -> InMemorySecretStore {
        let store = InMemorySecretStore::default();
        store.write(SecretKind::TsPassword, "Salve123").unwrap();
        store.write(SecretKind::TsPincode, "3489543096").unwrap();
        store
    }

    fn row(issue_date: &str, number: &str) -> TsReportRow {
        TsReportRow {
            vat_number: "65498732105".to_string(),
            issue_date: issue_date.to_string(),
            document_number: number.to_string(),
            payment_date: issue_date.to_string(),
            protocol: "99260926001866680".to_string(),
            sent_date: "2026-03-05".to_string(),
            send_kind: "I".to_string(),
            amount: 81.64,
            refunded_amount: 0.0,
            invoice_id: None,
        }
    }

    #[tokio::test]
    async fn never_sent_invoice_is_looked_up_by_its_own_id() {
        let db = setup().await;
        let gateway = FakeGateway::default();
        let result = query_invoice(&db, &store(), &gateway, 1).await.unwrap();
        assert_eq!(result, TsQueryResult::NotFound);
        assert_eq!(
            gateway.recorded(),
            vec![RecordedCall::Query(TsDocumentId {
                vat_number: "65498732105".to_string(),
                issue_date: "2026-03-01".to_string(),
                number: "1".to_string(),
            })]
        );
    }

    #[tokio::test]
    async fn sent_invoice_is_looked_up_by_the_id_it_was_accepted_with() {
        let db = setup().await;
        let gateway = FakeGateway::default();
        let sent = queue::enqueue_invio(&db, 1).await.unwrap();
        ts_dispatch_service::dispatch_due(&db, &store(), &gateway, at("2099-01-01 10:00:00"))
            .await
            .unwrap();
        let sent_id = ts_submission_repository::load(&db, sent.id)
            .await
            .unwrap()
            .document
            .unwrap();

        let document = TsRemoteDocument {
            id: sent_id.clone(),
            payment_date: Some("2026-03-02".to_string()),
            totals: vec![],
            refunded_totals: vec![],
            protocol: Some("99260926001866680".to_string()),
            sent_date: Some("2099-01-01".to_string()),
            send_kind: Some("I".to_string()),
            cancelled: false,
            messages: vec![],
        };
        *gateway.query_reply.lock().unwrap() = Some(TsQueryResult::Found {
            document: Box::new(document),
        });
        let TsQueryResult::Found { document } =
            query_invoice(&db, &store(), &gateway, 1).await.unwrap()
        else {
            panic!("expected the document");
        };
        assert_eq!(document.id, sent_id);
        assert_eq!(
            gateway.recorded().last(),
            Some(&RecordedCall::Query(sent_id))
        );
    }

    #[tokio::test]
    async fn monthly_report_links_rows_to_local_invoices() {
        let db = setup().await;
        let gateway = FakeGateway::default();
        *gateway.report_reply.lock().unwrap() = Some(ReportOutcome::Rows(vec![
            row("2026-03-01", "1"),
            row("2026-03-01", "FT-ALTRO"),
        ]));
        let rows = monthly_report(&db, &store(), &gateway, 2026, 3, TsReportBasis::Invio)
            .await
            .unwrap();
        assert_eq!(rows[0].invoice_id, Some(1));
        assert_eq!(rows[1].invoice_id, None);
        assert_eq!(
            gateway.recorded(),
            vec![RecordedCall::Report(2026, 3, TsReportBasis::Invio)]
        );
    }

    #[tokio::test]
    async fn monthly_report_surfaces_refusals_and_bad_periods() {
        let db = setup().await;
        let gateway = FakeGateway::default();
        *gateway.report_reply.lock().unwrap() = Some(ReportOutcome::Refused(vec![TsMessage {
            code: "WS46".to_string(),
            description: "TIPO ESTRAZIONE NON VALIDO".to_string(),
            kind: "E".to_string(),
        }]));
        let err = monthly_report(&db, &store(), &gateway, 2026, 3, TsReportBasis::Pagamento)
            .await
            .unwrap_err();
        assert!(err.contains("WS46"));
        assert!(
            monthly_report(&db, &store(), &gateway, 2026, 13, TsReportBasis::Invio)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn connection_check_reads_not_found_as_valid_credentials() {
        let db = setup().await;
        let gateway = FakeGateway::default();
        let check = check_connection(&db, &store(), &gateway).await.unwrap();
        assert!(check.ok);

        let missing = check_connection(&db, &InMemorySecretStore::default(), &gateway)
            .await
            .unwrap();
        assert!(!missing.ok);
        assert!(missing.message.contains("password"));
    }
}
