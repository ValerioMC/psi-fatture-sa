use serde::Serialize;

/// The user's acceptance of one version of the terms of use, with the specific approval of
/// the limitation clauses (artt. 1341-1342 c.c.).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TermsAcceptance {
    pub version: String,
    pub clauses_approved: bool,
    pub accepted_at: String,
}
