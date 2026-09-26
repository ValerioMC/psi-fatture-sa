use async_trait::async_trait;
use thiserror::Error;

use super::response::ReportOutcome;
use crate::app::model::ts::{
    TsCallResponse, TsDocumentId, TsEnvironment, TsExpenseDocument, TsQueryResult, TsReportBasis,
};

/// Everything a call needs to authenticate, in clear; it never leaves memory.
#[derive(Clone)]
pub struct TsSession {
    pub environment: TsEnvironment,
    pub username: String,
    pub password: String,
    pub pincode: String,
}

/// Redacts the secrets, so a session can be logged or asserted on safely.
impl std::fmt::Debug for TsSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TsSession")
            .field("environment", &self.environment)
            .field("username", &self.username)
            .field("password", &"***")
            .field("pincode", &"***")
            .finish()
    }
}

/// A call that produced no Sistema TS verdict. `Authentication` is the
/// HTTP-level refusal of the username/password pair.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TsGatewayError {
    #[error("Sistema TS non raggiungibile: {0}")]
    Network(String),
    #[error("Utente o password del Sistema TS non validi")]
    Authentication,
    #[error("Il Sistema TS ha rifiutato la richiesta: {0}")]
    Fault(String),
    #[error("Risposta inattesa dal Sistema TS (HTTP {0})")]
    Http(u16),
    #[error("Risposta del Sistema TS non leggibile: {0}")]
    Malformed(String),
    #[error("Preparazione della richiesta non riuscita: {0}")]
    Request(String),
}

/// The Sistema TS services the app uses: the synchronous document calls, the
/// point query and the monthly report.
#[async_trait]
pub trait SistemaTsGateway: Send + Sync {
    async fn insert(
        &self,
        session: &TsSession,
        document: &TsExpenseDocument,
    ) -> Result<TsCallResponse, TsGatewayError>;

    async fn update(
        &self,
        session: &TsSession,
        document: &TsExpenseDocument,
    ) -> Result<TsCallResponse, TsGatewayError>;

    async fn cancel(
        &self,
        session: &TsSession,
        id: &TsDocumentId,
    ) -> Result<TsCallResponse, TsGatewayError>;

    async fn query(
        &self,
        session: &TsSession,
        id: &TsDocumentId,
    ) -> Result<TsQueryResult, TsGatewayError>;

    async fn monthly_report(
        &self,
        session: &TsSession,
        year: i32,
        month: u32,
        basis: TsReportBasis,
    ) -> Result<ReportOutcome, TsGatewayError>;
}

#[cfg(test)]
pub mod testing {
    use std::collections::VecDeque;
    use std::sync::Mutex;

    use super::*;
    use crate::app::model::ts::{TsEsito, TsMessage};

    /// A call the fake received, with what identifies it.
    #[derive(Debug, Clone, PartialEq)]
    pub enum RecordedCall {
        Insert(TsExpenseDocument),
        Update(TsExpenseDocument),
        Cancel(TsDocumentId),
        Query(TsDocumentId),
        Report(i32, u32, TsReportBasis),
    }

    /// Answers document calls from a script, in order; accepts when the script is empty.
    #[derive(Default)]
    pub struct FakeGateway {
        pub calls: Mutex<Vec<RecordedCall>>,
        pub replies: Mutex<VecDeque<Result<TsCallResponse, TsGatewayError>>>,
        pub query_reply: Mutex<Option<TsQueryResult>>,
        pub report_reply: Mutex<Option<ReportOutcome>>,
    }

    impl FakeGateway {
        pub fn script(&self, reply: Result<TsCallResponse, TsGatewayError>) {
            self.replies.lock().unwrap().push_back(reply);
        }

        pub fn recorded(&self) -> Vec<RecordedCall> {
            self.calls.lock().unwrap().clone()
        }

        fn answer(&self, call: RecordedCall) -> Result<TsCallResponse, TsGatewayError> {
            self.calls.lock().unwrap().push(call);
            self.replies
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| Ok(accepted("99260926000000001")))
        }
    }

    pub fn accepted(protocol: &str) -> TsCallResponse {
        TsCallResponse {
            esito: TsEsito::Accepted,
            protocol: Some(protocol.to_string()),
            messages: vec![message("0", "")],
        }
    }

    pub fn rejected(code: &str) -> TsCallResponse {
        TsCallResponse {
            esito: TsEsito::Rejected,
            protocol: None,
            messages: vec![message(code, "E")],
        }
    }

    pub fn message(code: &str, kind: &str) -> TsMessage {
        TsMessage {
            code: code.to_string(),
            description: format!("descrizione {code}"),
            kind: kind.to_string(),
        }
    }

    #[async_trait]
    impl SistemaTsGateway for FakeGateway {
        async fn insert(
            &self,
            _session: &TsSession,
            document: &TsExpenseDocument,
        ) -> Result<TsCallResponse, TsGatewayError> {
            self.answer(RecordedCall::Insert(document.clone()))
        }

        async fn update(
            &self,
            _session: &TsSession,
            document: &TsExpenseDocument,
        ) -> Result<TsCallResponse, TsGatewayError> {
            self.answer(RecordedCall::Update(document.clone()))
        }

        async fn cancel(
            &self,
            _session: &TsSession,
            id: &TsDocumentId,
        ) -> Result<TsCallResponse, TsGatewayError> {
            self.answer(RecordedCall::Cancel(id.clone()))
        }

        async fn query(
            &self,
            _session: &TsSession,
            id: &TsDocumentId,
        ) -> Result<TsQueryResult, TsGatewayError> {
            self.calls
                .lock()
                .unwrap()
                .push(RecordedCall::Query(id.clone()));
            Ok(self
                .query_reply
                .lock()
                .unwrap()
                .clone()
                .unwrap_or(TsQueryResult::NotFound))
        }

        async fn monthly_report(
            &self,
            _session: &TsSession,
            year: i32,
            month: u32,
            basis: TsReportBasis,
        ) -> Result<ReportOutcome, TsGatewayError> {
            self.calls
                .lock()
                .unwrap()
                .push(RecordedCall::Report(year, month, basis));
            Ok(self
                .report_reply
                .lock()
                .unwrap()
                .take()
                .unwrap_or(ReportOutcome::Rows(Vec::new())))
        }
    }
}
