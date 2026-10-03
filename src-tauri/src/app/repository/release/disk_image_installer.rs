//! `BundleInstaller` over the macOS tools: `hdiutil` mounts the image, `ditto`
//! copies the bundle. Files written this way carry no quarantine attribute.

use std::path::Path;
use std::process::Command;

use super::BundleInstaller;
use crate::app::common::AppError;

pub struct DiskImageInstaller;

impl BundleInstaller for DiskImageInstaller {
    fn attach(&self, image: &Path, mount_point: &Path) -> Result<(), AppError> {
        run(
            Command::new("hdiutil")
                .args([
                    "attach",
                    "-nobrowse",
                    "-readonly",
                    "-noautoopen",
                    "-mountpoint",
                ])
                .arg(mount_point)
                .arg(image),
            "Apertura dell'immagine disco non riuscita",
        )
    }

    /// A busy volume is forced out: the update has already copied what it needed.
    fn detach(&self, mount_point: &Path) -> Result<(), AppError> {
        let quiet = run(
            Command::new("hdiutil")
                .args(["detach", "-quiet"])
                .arg(mount_point),
            "Chiusura dell'immagine disco non riuscita",
        );
        quiet.or_else(|_| {
            run(
                Command::new("hdiutil")
                    .args(["detach", "-force", "-quiet"])
                    .arg(mount_point),
                "Chiusura dell'immagine disco non riuscita",
            )
        })
    }

    fn copy_bundle(&self, source: &Path, target: &Path) -> Result<(), AppError> {
        run(
            Command::new("ditto").arg(source).arg(target),
            "Copia della nuova versione non riuscita",
        )
    }
}

fn run(command: &mut Command, failure: &str) -> Result<(), AppError> {
    let output = command
        .output()
        .map_err(|e| AppError::External(format!("{failure}: {e}")))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(AppError::External(format!("{failure}: {}", stderr.trim())))
}
