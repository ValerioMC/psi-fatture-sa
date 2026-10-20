use super::*;

#[test]
fn the_tag_page_and_assets_are_read_from_the_release_json() {
    let body = r#"{
        "tag_name": "v0.9.8",
        "html_url": "https://github.com/ValerioMC/psi-fatture-sa/releases/tag/v0.9.8",
        "draft": false,
        "assets": [
            {"name": "PSI-Fatture-macOS-arm64.dmg", "size": 1, "browser_download_url": "https://example.test/arm.dmg"}
        ]
    }"#;

    let release = parse_latest(body).unwrap();

    assert_eq!(release.tag, "v0.9.8");
    assert_eq!(
        release.page_url,
        "https://github.com/ValerioMC/psi-fatture-sa/releases/tag/v0.9.8"
    );
    assert_eq!(
        release.assets,
        vec![ReleaseAsset {
            name: "PSI-Fatture-macOS-arm64.dmg".to_string(),
            download_url: "https://example.test/arm.dmg".to_string(),
        }]
    );
}

#[test]
fn a_release_without_assets_has_none() {
    let release = parse_latest(r#"{"tag_name": "v1.0.0", "html_url": "https://x.test"}"#).unwrap();

    assert!(release.assets.is_empty());
}

#[test]
fn an_unreadable_answer_is_an_external_error() {
    assert!(matches!(
        parse_latest("<html>rate limited</html>"),
        Err(AppError::External(_))
    ));
}
