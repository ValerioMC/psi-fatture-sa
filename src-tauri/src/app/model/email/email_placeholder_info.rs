use serde::Serialize;

/// What the template editor shows for a placeholder: `{key}` and a human label.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EmailPlaceholderInfo {
    pub key: &'static str,
    pub label: &'static str,
}
