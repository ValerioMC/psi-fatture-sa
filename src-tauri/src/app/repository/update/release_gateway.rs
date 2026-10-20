use std::path::Path;

use async_trait::async_trait;

use crate::app::common::AppError;
use crate::app::model::update::LatestRelease;

/// Where published releases are announced and their files downloaded.
#[async_trait]
pub trait ReleaseGateway: Send + Sync {
    async fn latest(&self) -> Result<LatestRelease, AppError>;

    /// Writes the file at `url` to `target`, replacing it.
    async fn download(&self, url: &str, target: &Path) -> Result<(), AppError>;
}
