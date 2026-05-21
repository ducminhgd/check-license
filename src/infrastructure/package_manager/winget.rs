use crate::application::ports::PackageLicenseProvider;

/// Stub provider for Windows. winget does not expose SPDX license data in a
/// machine-readable format; the knowledge base covers common Windows apps instead.
pub struct WingetLicenseProvider;

impl PackageLicenseProvider for WingetLicenseProvider {
    fn lookup_spdx(&self, _name: &str, _bundle_id: Option<&str>) -> Option<String> {
        None
    }
}
