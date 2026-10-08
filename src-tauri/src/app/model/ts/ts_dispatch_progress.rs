use serde::Serialize;

use super::TsDispatchSummary;

/// Where a pass over the queue stands: `processed` of `total` submissions
/// answered so far. `finished` marks the last report of the pass, also when
/// it stopped early.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct TsDispatchProgress {
    pub total: u32,
    pub processed: u32,
    pub finished: bool,
    pub summary: TsDispatchSummary,
}

impl TsDispatchProgress {
    pub fn started(total: usize, summary: &TsDispatchSummary) -> Self {
        Self {
            total: u32::try_from(total).unwrap_or(u32::MAX),
            processed: 0,
            finished: false,
            summary: summary.clone(),
        }
    }

    pub fn advance(&mut self, summary: &TsDispatchSummary) {
        self.processed = (self.processed + 1).min(self.total);
        self.summary = summary.clone();
    }

    pub fn finish(&mut self, summary: &TsDispatchSummary) {
        self.finished = true;
        self.summary = summary.clone();
    }
}
