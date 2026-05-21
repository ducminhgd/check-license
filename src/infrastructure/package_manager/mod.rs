use crate::application::ports::PackageLicenseProvider;

#[cfg(target_os = "macos")]
pub mod homebrew;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod winget;

/// Fallback provider that always returns None. Used in tests and as a safe default.
#[allow(dead_code)]
pub struct NullLicenseProvider;

impl PackageLicenseProvider for NullLicenseProvider {
    fn lookup_spdx(&self, _name: &str, _bundle_id: Option<&str>) -> Option<String> {
        None
    }
}
