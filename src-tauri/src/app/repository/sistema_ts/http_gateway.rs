//! `SistemaTsGateway` over HTTPS with preemptive Basic auth. The test host
//! presents a certificate from the Sogei test CA, bundled and trusted only
//! for that environment; production uses the public roots.

use std::time::Duration;

use async_trait::async_trait;

use super::cipher::{FieldCipher, SanitelCipher};
use super::envelope::{self, Credentials, DocumentCall};
use super::gateway::{SistemaTsGateway, TsGatewayError, TsSession};
use super::response::{self, ReportOutcome};
use crate::app::model::ts::{
    TsCallResponse, TsDocumentId, TsEnvironment, TsExpenseDocument, TsQueryResult, TsReportBasis,
};

const SOGEI_TEST_CA: &str = include_str!("../../../../resources/sistema_ts/SogeiTestCA.pem");
const AUTHENTICATION_FAULT: &str = "autenticazione";

const DOCUMENT_PATH: &str = "/DocumentoSpesa730pWeb/DocumentoSpesa730pPort";
const QUERY_PATH: &str = "/InterrogazionePuntuale730Web/InterrogazionePuntuale730Port";
const REPORT_PATH: &str = "/ReportMensile730Web/ReportMensilePort";

pub struct HttpSistemaTsGateway {
    cipher: Box<dyn FieldCipher>,
    test_client: reqwest::Client,
    production_client: reqwest::Client,
}

impl HttpSistemaTsGateway {
    pub fn new() -> Result<Self, String> {
        let test_ca = reqwest::Certificate::from_pem(SOGEI_TEST_CA.as_bytes())
            .map_err(|e| format!("CA di test Sogei non leggibile: {e}"))?;
        Ok(Self {
            cipher: Box::new(SanitelCipher::from_bundled_certificate()?),
            test_client: client_builder()
                .add_root_certificate(test_ca)
                .build()
                .map_err(|e| e.to_string())?,
            production_client: client_builder().build().map_err(|e| e.to_string())?,
        })
    }

    fn client(&self, environment: TsEnvironment) -> &reqwest::Client {
        match environment {
            TsEnvironment::Test => &self.test_client,
            TsEnvironment::Produzione => &self.production_client,
        }
    }

    async fn post(
        &self,
        session: &TsSession,
        path: &str,
        action: &str,
        body: String,
    ) -> Result<String, TsGatewayError> {
        let url = format!("{}{}", session.environment.base_url(), path);
        let response = self
            .client(session.environment)
            .post(url)
            .basic_auth(&session.username, Some(&session.password))
            .header("Content-Type", "text/xml; charset=utf-8")
            .header("SOAPAction", format!("\"{action}\""))
            .body(body)
            .send()
            .await
            .map_err(network_error)?;

        let status = response.status();
        let text = response.text().await.map_err(network_error)?;
        classify_http(status.as_u16(), text)
    }

    async fn document_call(
        &self,
        call: DocumentCall,
        session: &TsSession,
        document: &TsExpenseDocument,
    ) -> Result<TsCallResponse, TsGatewayError> {
        let body =
            envelope::document_request(call, &credentials(session), document, self.cipher.as_ref())
                .map_err(TsGatewayError::Request)?;
        let action = match call {
            DocumentCall::Insert => "inserimento.documentospesap730.sanita.finanze.it",
            DocumentCall::Update => "variazione.documentospesap730.sanita.finanze.it",
        };
        let xml = self.post(session, DOCUMENT_PATH, action, body).await?;
        response::call_response(&xml).map_err(TsGatewayError::Malformed)
    }
}

#[async_trait]
impl SistemaTsGateway for HttpSistemaTsGateway {
    async fn insert(
        &self,
        session: &TsSession,
        document: &TsExpenseDocument,
    ) -> Result<TsCallResponse, TsGatewayError> {
        self.document_call(DocumentCall::Insert, session, document)
            .await
    }

    async fn update(
        &self,
        session: &TsSession,
        document: &TsExpenseDocument,
    ) -> Result<TsCallResponse, TsGatewayError> {
        self.document_call(DocumentCall::Update, session, document)
            .await
    }

    async fn cancel(
        &self,
        session: &TsSession,
        id: &TsDocumentId,
    ) -> Result<TsCallResponse, TsGatewayError> {
        let body = envelope::cancel_request(&credentials(session), id, self.cipher.as_ref())
            .map_err(TsGatewayError::Request)?;
        let xml = self
            .post(
                session,
                DOCUMENT_PATH,
                "cancellazione.documentospesap730.sanita.finanze.it",
                body,
            )
            .await?;
        response::call_response(&xml).map_err(TsGatewayError::Malformed)
    }

    async fn query(
        &self,
        session: &TsSession,
        id: &TsDocumentId,
    ) -> Result<TsQueryResult, TsGatewayError> {
        let body = envelope::query_request(&credentials(session), id, self.cipher.as_ref())
            .map_err(TsGatewayError::Request)?;
        let xml = self
            .post(
                session,
                QUERY_PATH,
                "interrogazionepuntuale.p730.sanita.finanze.it",
                body,
            )
            .await?;
        response::query_response(&xml).map_err(TsGatewayError::Malformed)
    }

    async fn monthly_report(
        &self,
        session: &TsSession,
        year: i32,
        month: u32,
        basis: TsReportBasis,
    ) -> Result<ReportOutcome, TsGatewayError> {
        let body = envelope::monthly_report_request(
            &credentials(session),
            year,
            month,
            basis,
            self.cipher.as_ref(),
        )
        .map_err(TsGatewayError::Request)?;
        let xml = self
            .post(
                session,
                REPORT_PATH,
                "reportmensile.p730.sanita.finanze.it",
                body,
            )
            .await?;
        response::report_response(&xml).map_err(TsGatewayError::Malformed)
    }
}

/// The whole cause chain: reqwest's own message rarely says what went wrong.
fn network_error(error: reqwest::Error) -> TsGatewayError {
    let mut message = error.to_string();
    let mut source = std::error::Error::source(&error);
    while let Some(cause) = source {
        message.push_str(&format!(": {cause}"));
        source = cause.source();
    }
    TsGatewayError::Network(message)
}

fn client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(60))
}

fn credentials(session: &TsSession) -> Credentials<'_> {
    Credentials {
        username: &session.username,
        pincode: &session.pincode,
    }
}

/// Faults come back as HTTP 500; the authentication one is the only fault
/// that is about the caller rather than the request.
fn classify_http(status: u16, body: String) -> Result<String, TsGatewayError> {
    if let Some(fault) = response::fault_message(&body) {
        if fault.to_lowercase().contains(AUTHENTICATION_FAULT) {
            return Err(TsGatewayError::Authentication);
        }
        return Err(TsGatewayError::Fault(fault));
    }
    match status {
        200 => Ok(body),
        401 | 403 => Err(TsGatewayError::Authentication),
        other => Err(TsGatewayError::Http(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAULT: &str = "<env:Envelope xmlns:env=\"http://schemas.xmlsoap.org/soap/envelope/\"><env:Body><env:Fault><faultcode>env:Client</faultcode><faultstring>{}</faultstring></env:Fault></env:Body></env:Envelope>";

    #[test]
    fn classifies_authentication_and_other_faults() {
        let auth = FAULT.replace("{}", "Errore generico di autenticazione");
        assert_eq!(
            classify_http(500, auth),
            Err(TsGatewayError::Authentication)
        );

        let internal = FAULT.replace("{}", "Internal Error");
        assert_eq!(
            classify_http(500, internal),
            Err(TsGatewayError::Fault("Internal Error".to_string()))
        );
    }

    #[test]
    fn passes_ok_bodies_and_maps_bare_statuses() {
        assert_eq!(
            classify_http(200, "<ok/>".to_string()),
            Ok("<ok/>".to_string())
        );
        assert_eq!(
            classify_http(401, String::new()),
            Err(TsGatewayError::Authentication)
        );
        assert_eq!(
            classify_http(503, String::new()),
            Err(TsGatewayError::Http(503))
        );
    }

    #[test]
    fn builds_with_the_bundled_certificates() {
        assert!(HttpSistemaTsGateway::new().is_ok());
    }
}

/// Round trip against the Sogei test environment with the public credentials
/// of the development kit (psychologist profile). Needs the network:
/// `cargo test live_ -- --ignored`.
#[cfg(test)]
mod live_tests {
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
}
