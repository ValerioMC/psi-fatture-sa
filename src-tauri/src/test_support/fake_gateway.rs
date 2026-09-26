use std::collections::VecDeque;
use std::sync::Mutex;

use async_trait::async_trait;

use super::ts_call_responses::accepted;
use super::RecordedCall;
use crate::app::model::ts::{
    TsCallResponse, TsDocumentId, TsExpenseDocument, TsQueryResult, TsReportBasis,
};
use crate::app::repository::ts::sistema_ts::{
    ReportOutcome, SistemaTsGateway, TsGatewayError, TsSession,
};

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
