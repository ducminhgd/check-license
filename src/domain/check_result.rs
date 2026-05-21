use crate::domain::{app_entry::AppEntry, license::{ActivationStatus, LicenseModel}};

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub entry: AppEntry,
    pub license_model: LicenseModel,
    /// Specific SPDX identifier (e.g. "MIT", "GPL-2.0-only", "Proprietary") when known.
    /// Falls back to the model name in the UI when None.
    pub spdx: Option<String>,
    pub activation_status: ActivationStatus,
    pub work_allowed: bool,
    pub crack_suspected: bool,
    pub notes: Vec<String>,
}
