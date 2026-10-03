use std::path::Path;

use crate::app::common::AppError;

/// The system tools an update needs to unpack a disk image and copy the app out of it.
pub trait BundleInstaller: Send + Sync {
    /// Mounts `image` read-only at `mount_point`, without opening a Finder window.
    fn attach(&self, image: &Path, mount_point: &Path) -> Result<(), AppError>;

    fn detach(&self, mount_point: &Path) -> Result<(), AppError>;

    /// Copies an `.app` bundle with its signature, permissions and symlinks intact.
    fn copy_bundle(&self, source: &Path, target: &Path) -> Result<(), AppError>;
}
