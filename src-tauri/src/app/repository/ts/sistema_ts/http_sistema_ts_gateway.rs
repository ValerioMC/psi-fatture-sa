//! `SistemaTsGateway` over HTTPS with preemptive Basic auth. The test host
//! presents a certificate from the Sogei test CA, bundled and trusted only
//! for that environment; production uses the public roots.

use std::time::Duration;

use async_trait::async_trait;

use super::{
    envelope, response, Credentials, DocumentCall, FieldCipher, ReportOutcome, SanitelCipher,
    SistemaTsGateway, TsGatewayError, TsSession,
};
use crate::app::model::ts::{
    TsCallResponse, TsDocumentId, TsEnvironment, TsExpenseDocument, TsQueryResult, TsReportBasis,
};

const SOGEI_TEST_CA: &str = include_str!("../../../../../resources/sistema_ts/SogeiTestCA.pem");
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
        if !session.environment.is_available() {
            return Err(TsGatewayError::UnavailableEnvironment);
        }
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
#[path = "http_sistema_ts_gateway_test.rs"]
mod tests;

#[cfg(test)]
#[path = "http_sistema_ts_gateway_live_test.rs"]
mod live_tests;
