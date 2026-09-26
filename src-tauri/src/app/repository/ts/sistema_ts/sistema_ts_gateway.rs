use async_trait::async_trait;

use super::{ReportOutcome, TsGatewayError, TsSession};
use crate::app::model::ts::{
    TsCallResponse, TsDocumentId, TsExpenseDocument, TsQueryResult, TsReportBasis,
};

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
