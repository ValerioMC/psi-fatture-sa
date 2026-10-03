//! Installs the published release over a stand-in bundle in a temp folder, with the
//! real GitHub download, `hdiutil` and `ditto`. Mac and network only:
//! `cargo test live_ -- --ignored`.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::*;
use crate::app::repository::release::{DiskImageInstaller, GithubReleaseGateway};

fn bundle_in(root: &Path) -> PathBuf {
    let bundle = root.join("Applications").join("PSI Fatture.app");
    std::fs::create_dir_all(bundle.join("Contents").join("MacOS")).unwrap();
    bundle
}

#[tokio::test]
#[ignore]
async fn live_the_published_release_installs_without_quarantine() {
    let root = std::env::temp_dir().join(format!("psi-install-live-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let bundle = bundle_in(&root);
    let gateway = GithubReleaseGateway::new("0.0.1").unwrap();
    let platform = Platform::current();

    let version = install(UpdateTarget {
        gateway: &gateway,
        installer: &DiskImageInstaller,
        current_version: "0.0.1",
        platform,
        running_executable: &bundle.join("Contents").join("MacOS").join("psi-fatture-sa"),
        work_dir: &root.join("work"),
    })
    .await
    .unwrap();

    assert!(
        bundle.join("Contents").join("Info.plist").is_file(),
        "{version}"
    );
    let attributes = Command::new("xattr")
        .arg("-r")
        .arg(&bundle)
        .output()
        .unwrap();
    let listed = String::from_utf8_lossy(&attributes.stdout);
    assert!(!listed.contains("com.apple.quarantine"), "{listed}");
    std::fs::remove_dir_all(&root).unwrap();
}
