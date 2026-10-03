use serde::Serialize;

/// The system the app runs on, as far as choosing a download goes.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    MacOsArm64,
    MacOsX64,
    WindowsX64,
    /// No bundle is published for it: the release page is the only link.
    Other,
}

impl Platform {
    pub fn current() -> Self {
        Self::of(std::env::consts::OS, std::env::consts::ARCH)
    }

    /// `os` and `arch` as `std::env::consts` spells them.
    pub fn of(os: &str, arch: &str) -> Self {
        match (os, arch) {
            ("macos", "aarch64") => Platform::MacOsArm64,
            ("macos", "x86_64") => Platform::MacOsX64,
            ("windows", "x86_64") => Platform::WindowsX64,
            _ => Platform::Other,
        }
    }

    /// The stable file name the release workflow uploads for this platform.
    pub fn asset_name(self) -> Option<&'static str> {
        match self {
            Platform::MacOsArm64 => Some("PSI-Fatture-macOS-arm64.dmg"),
            Platform::MacOsX64 => Some("PSI-Fatture-macOS-x64.dmg"),
            Platform::WindowsX64 => Some("PSI-Fatture-Windows-x64-setup.exe"),
            Platform::Other => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Platform::MacOsArm64 => "Mac con chip Apple",
            Platform::MacOsX64 => "Mac con processore Intel",
            Platform::WindowsX64 => "Windows",
            Platform::Other => "questo sistema",
        }
    }
}

#[cfg(test)]
#[path = "platform_test.rs"]
mod tests;
