use crate::app::model::ts::{TsMessage, TsReportRow};

/// The monthly report: its rows, or the messages it was refused with.
#[derive(Debug, PartialEq)]
pub enum ReportOutcome {
    Rows(Vec<TsReportRow>),
    Refused(Vec<TsMessage>),
}
