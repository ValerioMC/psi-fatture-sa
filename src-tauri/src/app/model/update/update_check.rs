use serde::Serialize;

use super::Platform;

/// The outcome of comparing this build with the newest release. `download_url` is the
/// bundle for `platform` or, when none is published for it, the release page.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UpdateCheck {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: String,
    pub platform: Platform,
    pub platform_label: String,
}
