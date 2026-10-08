//! Carries the progress of a Sistema TS pass to the window as a Tauri event,
//! whether the pass was asked for by the user or run by the worker.

use tauri::{AppHandle, Emitter};

use crate::app::model::ts::TsDispatchProgress;

pub const TS_DISPATCH_PROGRESS_EVENT: &str = "ts-dispatch-progress";

/// A lost report only leaves the bar behind; the pass itself goes on.
pub fn emitter(app: AppHandle) -> impl Fn(&TsDispatchProgress) + Send + Sync {
    move |progress| {
        if let Err(error) = app.emit(TS_DISPATCH_PROGRESS_EVENT, progress) {
            log::warn!(target: "sistema_ts", error:% = error; "progress event not delivered");
        }
    }
}
