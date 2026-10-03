//! Compares this build with the newest published release and picks the
//! download that fits the system the app is running on.

use crate::app::common::AppError;
use crate::app::model::update::{LatestRelease, Platform, ReleaseVersion, UpdateCheck};
use crate::app::repository::release::ReleaseGateway;

pub async fn check(
    gateway: &dyn ReleaseGateway,
    current_version: &str,
    platform: Platform,
) -> Result<UpdateCheck, AppError> {
    let release = gateway.latest().await?;
    evaluate(&release, current_version, platform)
}

/// Compares `release` with this build; a tag that is not a version is the server's fault.
pub fn evaluate(
    release: &LatestRelease,
    current_version: &str,
    platform: Platform,
) -> Result<UpdateCheck, AppError> {
    let current = ReleaseVersion::parse(current_version)?;
    let latest = ReleaseVersion::parse(&release.tag).map_err(|_| {
        AppError::External(format!(
            "L'ultima versione pubblicata ha un numero non valido: '{}'",
            release.tag
        ))
    })?;
    Ok(UpdateCheck {
        current_version: current.to_string(),
        latest_version: latest.to_string(),
        update_available: latest > current,
        download_url: bundle_url(release, platform).unwrap_or_else(|| release.page_url.clone()),
        platform,
        platform_label: platform.label().to_string(),
    })
}

/// The bundle published for `platform`, if the release has one.
pub fn bundle_url(release: &LatestRelease, platform: Platform) -> Option<String> {
    platform
        .asset_name()
        .and_then(|wanted| release.assets.iter().find(|asset| asset.name == wanted))
        .map(|asset| asset.download_url.clone())
}

#[cfg(test)]
#[path = "update_service_test.rs"]
mod tests;
