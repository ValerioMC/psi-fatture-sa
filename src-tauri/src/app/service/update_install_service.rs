//! Installs the newest release over the running app on macOS. The app downloads
//! the image itself, so no quarantine attribute is set and the new version opens
//! without `xattr`. The old bundle is kept aside until the new one is in place.

use std::path::{Path, PathBuf};

use crate::app::common::AppError;
use crate::app::model::update::Platform;
use crate::app::repository::release::{bundle_file_repository, BundleInstaller, ReleaseGateway};
use crate::app::service::update_service;

/// Only files of this repository's releases are ever installed.
pub const DOWNLOAD_PREFIX: &str = "https://github.com/ValerioMC/psi-fatture-sa/releases/download/";
const IMAGE_NAME: &str = "PSI-Fatture-update.dmg";
const MOUNT_NAME: &str = "volume";

/// What the update acts on: the tools, this build and where it may write scratch files.
pub struct UpdateTarget<'a> {
    pub gateway: &'a dyn ReleaseGateway,
    pub installer: &'a dyn BundleInstaller,
    pub current_version: &'a str,
    pub platform: Platform,
    pub running_executable: &'a Path,
    pub work_dir: &'a Path,
}

/// Returns the installed version. Nothing is downloaded unless a newer bundle for this Mac exists.
pub async fn install(target: UpdateTarget<'_>) -> Result<String, AppError> {
    if !matches!(target.platform, Platform::MacOsArm64 | Platform::MacOsX64) {
        return Err(AppError::Invalid(
            "L'installazione dall'app è disponibile solo su Mac".to_string(),
        ));
    }
    let bundle = app_bundle_of(target.running_executable)?;
    let release = target.gateway.latest().await?;
    let check = update_service::evaluate(&release, target.current_version, target.platform)?;
    if !check.update_available {
        return Err(AppError::Conflict(format!(
            "Hai già l'ultima versione ({})",
            check.current_version
        )));
    }
    let url = update_service::bundle_url(&release, target.platform)
        .filter(|url| url.starts_with(DOWNLOAD_PREFIX))
        .ok_or_else(|| {
            AppError::External(format!(
                "La versione {} non ha un pacchetto per {}",
                check.latest_version,
                target.platform.label()
            ))
        })?;

    bundle_file_repository::reset_folder(target.work_dir)?;
    let image = target.work_dir.join(IMAGE_NAME);
    target.gateway.download(&url, &image).await?;
    let mount_point = target.work_dir.join(MOUNT_NAME);
    bundle_file_repository::reset_folder(&mount_point)?;
    target.installer.attach(&image, &mount_point)?;
    let replaced = bundle_file_repository::find_app_in(&mount_point)
        .and_then(|source| replace_bundle(target.installer, &source, &bundle));
    if let Err(error) = target.installer.detach(&mount_point) {
        log::warn!(target: "update", error:% = error; "disk image left mounted");
    }
    if let Err(error) = bundle_file_repository::remove_tree(&image) {
        log::warn!(target: "update", error:% = error; "downloaded image not deleted");
    }
    replaced?;
    log::info!(target: "update", version:% = check.latest_version, bundle:? = bundle; "update installed");
    Ok(check.latest_version)
}

/// The `.app` directory the executable runs from; a development binary has none.
pub fn app_bundle_of(executable: &Path) -> Result<PathBuf, AppError> {
    executable
        .ancestors()
        .find(|path| path.extension().is_some_and(|ext| ext == "app"))
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            AppError::Conflict(
                "L'app non è in esecuzione da un pacchetto installato: aggiornala a mano"
                    .to_string(),
            )
        })
}

/// Copies the new bundle beside the old, then swaps names. A failed swap puts the old one back.
fn replace_bundle(
    installer: &dyn BundleInstaller,
    source: &Path,
    bundle: &Path,
) -> Result<(), AppError> {
    let staging = sibling(bundle, "new");
    let previous = sibling(bundle, "old");
    bundle_file_repository::remove_tree(&staging)?;
    if let Err(error) = installer.copy_bundle(source, &staging) {
        bundle_file_repository::remove_tree(&staging)?;
        return Err(error);
    }
    bundle_file_repository::remove_tree(&previous)?;
    bundle_file_repository::rename(bundle, &previous)?;
    if let Err(error) = bundle_file_repository::rename(&staging, bundle) {
        bundle_file_repository::rename(&previous, bundle)?;
        return Err(error);
    }
    if let Err(error) = bundle_file_repository::remove_tree(&previous) {
        log::warn!(target: "update", error:% = error; "previous version not deleted");
    }
    Ok(())
}

/// `/Applications/PSI Fatture.app` → `/Applications/.PSI Fatture.app.new`, hidden in Finder.
fn sibling(bundle: &Path, suffix: &str) -> PathBuf {
    let name = bundle.file_name().unwrap_or_default().to_string_lossy();
    bundle.with_file_name(format!(".{name}.{suffix}"))
}

#[cfg(test)]
#[path = "update_install_service_test.rs"]
mod tests;

#[cfg(test)]
#[path = "update_install_live_test.rs"]
mod live_tests;
