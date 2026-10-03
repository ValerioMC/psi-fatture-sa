use async_trait::async_trait;

use crate::app::common::AppError;
use crate::app::model::update::LatestRelease;

/// Where published releases are announced.
#[async_trait]
pub trait ReleaseGateway: Send + Sync {
    async fn latest(&self) -> Result<LatestRelease, AppError>;
}
