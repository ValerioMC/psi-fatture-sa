use super::*;
use crate::app::model::ts::{TsMessage, TsRemoteDocument};
use crate::app::repository::secret::SecretKind;
use crate::app::service::ts::ts_dispatch_service;
use crate::app::service::ts::ts_submission_service as queue;
use crate::test_support::clock::at;
use crate::test_support::ts_database::setup;
use crate::test_support::{FakeGateway, InMemorySecretStore, RecordedCall};

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
