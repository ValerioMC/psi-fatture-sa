use serde::{Deserialize, Serialize};

/// The subject and plain-text body every invoice email starts from, with
/// `{placeholders}` filled in per invoice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmailTemplate {
    pub subject: String,
    pub body: String,
}
