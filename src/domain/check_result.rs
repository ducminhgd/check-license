use crate::domain::{app_entry::AppEntry, license::{ActivationStatus, LicenseModel}};

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub entry: AppEntry,
    pub license_model: LicenseModel,
    pub activation_status: ActivationStatus,
    pub work_allowed: bool,
    pub crack_suspected: bool,
    pub notes: Vec<String>,
}
