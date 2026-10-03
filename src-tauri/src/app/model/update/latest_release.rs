use super::ReleaseAsset;

/// The newest published release: its tag as written (`v0.9.8`), its page and its files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatestRelease {
    pub tag: String,
    pub page_url: String,
    pub assets: Vec<ReleaseAsset>,
}
