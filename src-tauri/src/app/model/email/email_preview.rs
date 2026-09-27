use serde::Serialize;

/// A template with its placeholders filled in.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EmailPreview {
    pub subject: String,
    pub body: String,
}
