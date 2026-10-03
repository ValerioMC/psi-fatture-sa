use async_trait::async_trait;

use super::*;
use crate::app::model::update::ReleaseAsset;

struct FixedRelease(Result<LatestRelease, String>);

#[async_trait]
impl ReleaseGateway for FixedRelease {
    async fn latest(&self) -> Result<LatestRelease, AppError> {
        self.0.clone().map_err(AppError::External)
    }
}

fn published(tag: &str) -> FixedRelease {
    FixedRelease(Ok(LatestRelease {
        tag: tag.to_string(),
        page_url: "https://releases.test/page".to_string(),
        assets: vec![
            ReleaseAsset {
                name: "PSI-Fatture-macOS-arm64.dmg".to_string(),
                download_url: "https://releases.test/arm.dmg".to_string(),
            },
            ReleaseAsset {
                name: "PSI-Fatture-Windows-x64-setup.exe".to_string(),
                download_url: "https://releases.test/setup.exe".to_string(),
            },
        ],
    }))
}

#[tokio::test]
async fn a_newer_release_offers_the_bundle_for_this_system() {
    let check = check(&published("v0.9.8"), "0.9.7", Platform::WindowsX64)
        .await
        .unwrap();

    assert!(check.update_available);
    assert_eq!(check.current_version, "0.9.7");
    assert_eq!(check.latest_version, "0.9.8");
    assert_eq!(check.download_url, "https://releases.test/setup.exe");
    assert_eq!(check.platform_label, "Windows");
}

#[tokio::test]
async fn the_same_version_is_not_an_update() {
    let check = check(&published("v0.9.8"), "0.9.8", Platform::MacOsArm64)
        .await
        .unwrap();

    assert!(!check.update_available);
}

#[tokio::test]
async fn an_older_release_is_not_an_update() {
    let check = check(&published("v0.9.8"), "0.10.0", Platform::MacOsArm64)
        .await
        .unwrap();

    assert!(!check.update_available);
}

#[tokio::test]
async fn a_platform_without_its_bundle_gets_the_release_page() {
    let missing = check(&published("v1.0.0"), "0.9.8", Platform::MacOsX64)
        .await
        .unwrap();
    let unsupported = check(&published("v1.0.0"), "0.9.8", Platform::Other)
        .await
        .unwrap();

    assert_eq!(missing.download_url, "https://releases.test/page");
    assert_eq!(unsupported.download_url, "https://releases.test/page");
}

#[tokio::test]
async fn an_unreachable_server_is_an_external_error() {
    let offline = FixedRelease(Err("offline".to_string()));

    let error = check(&offline, "0.9.8", Platform::MacOsArm64)
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::External(_)));
}

#[tokio::test]
async fn a_malformed_release_tag_is_an_external_error() {
    let error = check(&published("nightly"), "0.9.8", Platform::MacOsArm64)
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::External(_)));
}
