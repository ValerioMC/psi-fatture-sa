use serde::Deserialize;

/// The two separate checkboxes of the terms screen, for the version it showed.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AcceptTermsInput {
    pub version: String,
    pub terms_accepted: bool,
    pub clauses_approved: bool,
}
