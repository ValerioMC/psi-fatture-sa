pub mod bundle_file_repository;
pub mod bundle_installer;
pub mod disk_image_installer;
pub mod github_release_gateway;
pub mod release_gateway;

pub use bundle_installer::BundleInstaller;
pub use disk_image_installer::DiskImageInstaller;
pub use github_release_gateway::GithubReleaseGateway;
pub use release_gateway::ReleaseGateway;
