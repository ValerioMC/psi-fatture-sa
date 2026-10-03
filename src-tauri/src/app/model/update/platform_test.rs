use super::*;

#[test]
fn each_published_bundle_has_its_platform() {
    assert_eq!(Platform::of("macos", "aarch64"), Platform::MacOsArm64);
    assert_eq!(Platform::of("macos", "x86_64"), Platform::MacOsX64);
    assert_eq!(Platform::of("windows", "x86_64"), Platform::WindowsX64);
}

#[test]
fn a_system_without_a_bundle_has_no_asset() {
    let platform = Platform::of("linux", "x86_64");

    assert_eq!(platform, Platform::Other);
    assert_eq!(platform.asset_name(), None);
}
