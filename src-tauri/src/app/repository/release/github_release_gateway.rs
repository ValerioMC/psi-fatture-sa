//! Reads the newest published release of the app from the GitHub API, the same
//! source the showcase site's download buttons resolve against.

use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;

use super::ReleaseGateway;
use crate::app::common::AppError;
use crate::app::model::update::{LatestRelease, ReleaseAsset};

const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/ValerioMC/psi-fatture-sa/releases/latest";
const TIMEOUT: Duration = Duration::from_secs(10);

pub struct GithubReleaseGateway {
    client: reqwest::Client,
}

impl GithubReleaseGateway {
    /// GitHub refuses API calls without a User-Agent, so the app's version goes in it.
    pub fn new(app_version: &str) -> Result<Self, AppError> {
        let client = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .user_agent(format!("PSI-Fatture/{app_version}"))
            .build()
            .map_err(|e| AppError::External(format!("Client HTTP non disponibile: {e}")))?;
        Ok(Self { client })
    }
}

#[async_trait]
impl ReleaseGateway for GithubReleaseGateway {
    async fn latest(&self) -> Result<LatestRelease, AppError> {
        let response = self
            .client
            .get(LATEST_RELEASE_URL)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| unreachable_error(&e.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            return Err(unreachable_error(&format!("risposta HTTP {status}")));
        }
        let body = response
            .text()
            .await
            .map_err(|e| unreachable_error(&e.to_string()))?;
        parse_latest(&body)
    }
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

/// The fields of GitHub's release JSON the update check needs; the rest is ignored.
pub fn parse_latest(body: &str) -> Result<LatestRelease, AppError> {
    let release: GithubRelease = serde_json::from_str(body).map_err(|e| {
        AppError::External(format!("Risposta sugli aggiornamenti non leggibile: {e}"))
    })?;
    Ok(LatestRelease {
        tag: release.tag_name,
        page_url: release.html_url,
        assets: release
            .assets
            .into_iter()
            .map(|asset| ReleaseAsset {
                name: asset.name,
                download_url: asset.browser_download_url,
            })
            .collect(),
    })
}

fn unreachable_error(cause: &str) -> AppError {
    AppError::External(format!(
        "Impossibile verificare gli aggiornamenti ({cause})"
    ))
}

#[cfg(test)]
#[path = "github_release_gateway_test.rs"]
mod tests;
