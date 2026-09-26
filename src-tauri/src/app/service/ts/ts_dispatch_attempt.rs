/// How one submission of a dispatch pass ended; `StopPass` ends the pass too.
pub(super) enum TsDispatchAttempt {
    Accepted,
    Rejected,
    Retrying,
    StopPass(String),
    Skipped,
}
