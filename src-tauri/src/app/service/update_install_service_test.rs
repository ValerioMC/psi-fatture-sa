use std::sync::Mutex;

use async_trait::async_trait;

use super::*;
use crate::app::model::update::{LatestRelease, ReleaseAsset};

const ARM_URL: &str =
    "https://github.com/ValerioMC/psi-fatture-sa/releases/download/v0.9.8/PSI-Fatture-macOS-arm64.dmg";

/// Serves one release; `download` writes the URL into the file so tests can see what was fetched.
struct FakeRelease {
    tag: String,
    url: String,
    downloads: Mutex<Vec<String>>,
}

impl FakeRelease {
    fn new(tag: &str, url: &str) -> Self {
        Self {
            tag: tag.to_string(),
            url: url.to_string(),
            downloads: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl ReleaseGateway for FakeRelease {
    async fn latest(&self) -> Result<LatestRelease, AppError> {
        Ok(LatestRelease {
            tag: self.tag.clone(),
            page_url: "https://github.com/ValerioMC/psi-fatture-sa/releases/tag/v0.9.8".to_string(),
            assets: vec![ReleaseAsset {
                name: "PSI-Fatture-macOS-arm64.dmg".to_string(),
                download_url: self.url.clone(),
            }],
        })
    }

    async fn download(&self, url: &str, target: &Path) -> Result<(), AppError> {
        self.downloads.lock().unwrap().push(url.to_string());
        std::fs::write(target, url).map_err(AppError::from)
    }
}

/// "Mounts" an image by creating `PSI Fatture.app` with a version marker inside the mount point.
struct FakeInstaller {
    copy_fails: bool,
    detached: Mutex<bool>,
}

impl FakeInstaller {
    fn new() -> Self {
        Self {
            copy_fails: false,
            detached: Mutex::new(false),
        }
    }
}

impl BundleInstaller for FakeInstaller {
    fn attach(&self, image: &Path, mount_point: &Path) -> Result<(), AppError> {
        assert!(image.is_file());
        let app = mount_point.join("PSI Fatture.app");
        std::fs::create_dir_all(&app)?;
        std::fs::write(app.join("version"), "new")?;
        Ok(())
    }

    fn detach(&self, _mount_point: &Path) -> Result<(), AppError> {
        *self.detached.lock().unwrap() = true;
        Ok(())
    }

    fn copy_bundle(&self, source: &Path, target: &Path) -> Result<(), AppError> {
        if self.copy_fails {
            return Err(AppError::External("disk full".to_string()));
        }
        std::fs::create_dir_all(target)?;
        std::fs::copy(source.join("version"), target.join("version"))?;
        Ok(())
    }
}

struct Scratch {
    root: PathBuf,
    bundle: PathBuf,
    executable: PathBuf,
    work_dir: PathBuf,
}

fn scratch(name: &str) -> Scratch {
    let root = std::env::temp_dir().join(format!("psi-install-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let bundle = root.join("Applications").join("PSI Fatture.app");
    let executable = bundle.join("Contents").join("MacOS").join("psi-fatture-sa");
    std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
    std::fs::write(bundle.join("version"), "old").unwrap();
    Scratch {
        work_dir: root.join("work"),
        root,
        bundle,
        executable,
    }
}

fn target<'a>(
    gateway: &'a FakeRelease,
    installer: &'a FakeInstaller,
    scratch: &'a Scratch,
    platform: Platform,
) -> UpdateTarget<'a> {
    UpdateTarget {
        gateway,
        installer,
        current_version: "0.9.7",
        platform,
        running_executable: &scratch.executable,
        work_dir: &scratch.work_dir,
    }
}

fn installed_version(scratch: &Scratch) -> String {
    std::fs::read_to_string(scratch.bundle.join("version")).unwrap()
}

#[tokio::test]
async fn the_new_bundle_replaces_the_running_one() {
    let scratch = scratch("replace");
    let gateway = FakeRelease::new("v0.9.8", ARM_URL);
    let installer = FakeInstaller::new();

    let version = install(target(&gateway, &installer, &scratch, Platform::MacOsArm64))
        .await
        .unwrap();

    assert_eq!(version, "0.9.8");
    assert_eq!(installed_version(&scratch), "new");
    assert_eq!(
        *gateway.downloads.lock().unwrap(),
        vec![ARM_URL.to_string()]
    );
    assert!(*installer.detached.lock().unwrap());
    let leftovers: Vec<String> = std::fs::read_dir(scratch.bundle.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(leftovers, vec!["PSI Fatture.app"]);
    assert!(!scratch.work_dir.join(IMAGE_NAME).exists());
    std::fs::remove_dir_all(&scratch.root).unwrap();
}

#[tokio::test]
async fn a_failed_copy_leaves_the_running_bundle_untouched() {
    let scratch = scratch("copy-fails");
    let gateway = FakeRelease::new("v0.9.8", ARM_URL);
    let installer = FakeInstaller {
        copy_fails: true,
        ..FakeInstaller::new()
    };

    let error = install(target(&gateway, &installer, &scratch, Platform::MacOsArm64))
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::External(_)));
    assert_eq!(installed_version(&scratch), "old");
    assert!(*installer.detached.lock().unwrap());
    assert!(!scratch
        .root
        .join("Applications")
        .join(".PSI Fatture.app.new")
        .exists());
    std::fs::remove_dir_all(&scratch.root).unwrap();
}

#[tokio::test]
async fn nothing_is_downloaded_when_already_up_to_date() {
    let scratch = scratch("current");
    let gateway = FakeRelease::new("v0.9.7", ARM_URL);
    let installer = FakeInstaller::new();

    let error = install(target(&gateway, &installer, &scratch, Platform::MacOsArm64))
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::Conflict(_)));
    assert!(gateway.downloads.lock().unwrap().is_empty());
    std::fs::remove_dir_all(&scratch.root).unwrap();
}

#[tokio::test]
async fn a_bundle_outside_this_repository_is_refused() {
    let scratch = scratch("foreign");
    let gateway = FakeRelease::new("v0.9.8", "https://example.test/PSI-Fatture-macOS-arm64.dmg");
    let installer = FakeInstaller::new();

    let error = install(target(&gateway, &installer, &scratch, Platform::MacOsArm64))
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::External(_)));
    assert!(gateway.downloads.lock().unwrap().is_empty());
    std::fs::remove_dir_all(&scratch.root).unwrap();
}

#[tokio::test]
async fn windows_is_refused_before_anything_happens() {
    let scratch = scratch("windows");
    let gateway = FakeRelease::new("v0.9.8", ARM_URL);
    let installer = FakeInstaller::new();

    let error = install(target(&gateway, &installer, &scratch, Platform::WindowsX64))
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::Invalid(_)));
    assert!(gateway.downloads.lock().unwrap().is_empty());
    std::fs::remove_dir_all(&scratch.root).unwrap();
}

#[test]
fn the_bundle_is_found_above_the_executable() {
    let executable = Path::new("/Applications/PSI Fatture.app/Contents/MacOS/psi-fatture-sa");

    assert_eq!(
        app_bundle_of(executable).unwrap(),
        PathBuf::from("/Applications/PSI Fatture.app")
    );
}

#[test]
fn a_development_binary_has_no_bundle() {
    let executable = Path::new("/repo/src-tauri/target/debug/psi-fatture-sa");

    assert!(matches!(
        app_bundle_of(executable),
        Err(AppError::Conflict(_))
    ));
}
