use serde::Serialize;

use super::{TsMessage, TsRemoteDocument};

/// Outcome of looking one document up on the Sistema TS.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TsQueryResult {
    Found { document: Box<TsRemoteDocument> },
    NotFound,
    Refused { messages: Vec<TsMessage> },
}
