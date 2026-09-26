//! Round trip against the Sogei test environment with the public credentials
//! of the development kit (psychologist profile). Needs the network:
//! `cargo test live_ -- --ignored`.

use super::*;
use crate::app::model::ts::{TsEsito, TsExpenseItem, TsVatTreatment};

const ATTEMPTS: usize = 5;

fn session() -> TsSession {
    TsSession {
        environment: TsEnvironment::Test,
        username: "MTOMRA66A41G224M".to_string(),
        password: "Salve123".to_string(),
        pincode: "3489543096".to_string(),
    }
}

fn document(number: &str, amount: f64) -> TsExpenseDocument {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    TsExpenseDocument {
        id: TsDocumentId {
            vat_number: "65498732105".to_string(),
            issue_date: today.clone(),
            number: number.to_string(),
        },
        payment_date: today,
        citizen_fiscal_code: Some("RSSMRA80A01H501U".to_string()),
        items: vec![TsExpenseItem {
            amount,
            vat: TsVatTreatment::Natura("N2.2".to_string()),
        }],
        traced_payment: true,
    }
}

/// The test environment answers WS99 now and then: retry like the queue does.
async fn until_settled<F, Fut>(call: F) -> TsCallResponse
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<TsCallResponse, TsGatewayError>>,
{
    let mut last = None;
    for _ in 0..ATTEMPTS {
        let response = call().await.expect("call failed");
        if !response.has_code("WS99") {
            return response;
        }
        last = Some(response);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    last.expect("no attempt made")
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "calls the Sistema TS test environment"]
async fn live_insert_query_update_report_cancel() {
    let gateway = HttpSistemaTsGateway::new().unwrap();
    let session = session();
    let number = format!("PSI{}", chrono::Local::now().format("%H%M%S"));
    let original = document(&number, 80.0);

    let inserted = until_settled(|| gateway.insert(&session, &original)).await;
    assert!(inserted.accepted(), "insert: {inserted:?}");
    assert_eq!(inserted.protocol.as_ref().map(String::len), Some(17));

    let TsQueryResult::Found { document: found } =
        gateway.query(&session, &original.id).await.unwrap()
    else {
        panic!("inserted document not found");
    };
    assert_eq!(found.totals[0].amount, 80.0);

    let changed = document(&number, 95.5);
    let updated = until_settled(|| gateway.update(&session, &changed)).await;
    assert!(updated.accepted(), "update: {updated:?}");

    let now = chrono::Local::now();
    use chrono::Datelike;
    let ReportOutcome::Rows(rows) = gateway
        .monthly_report(&session, now.year(), now.month(), TsReportBasis::Invio)
        .await
        .unwrap()
    else {
        panic!("report refused");
    };
    let row = rows
        .iter()
        .find(|r| r.document_number == number)
        .expect("row in report");
    assert_eq!(row.amount, 95.5);

    let cancelled = until_settled(|| gateway.cancel(&session, &original.id)).await;
    assert!(cancelled.accepted(), "cancel: {cancelled:?}");
    let TsQueryResult::Found { document: after } =
        gateway.query(&session, &original.id).await.unwrap()
    else {
        panic!("cancelled document should still be readable");
    };
    assert!(after.cancelled);

    let again = until_settled(|| gateway.cancel(&session, &original.id)).await;
    assert_eq!(again.esito, TsEsito::Rejected);
    assert!(again.has_code("S022"));
}

/// Observed on the test host: a wrong password gets a SOAP Fault for some
/// requests and no answer at all for a well-formed, encrypted one.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "calls the Sistema TS test environment"]
async fn live_wrong_password_never_reaches_a_verdict() {
    let gateway = HttpSistemaTsGateway::new().unwrap();
    let mut session = session();
    session.password = "sbagliata".to_string();
    let result = gateway.query(&session, &document("NOPE", 1.0).id).await;
    assert!(
        matches!(
            result,
            Err(TsGatewayError::Authentication | TsGatewayError::Network(_))
        ),
        "{result:?}"
    );
}
