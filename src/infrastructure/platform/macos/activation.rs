use crate::{
    application::ports::ActivationChecker,
    domain::{app_entry::AppEntry, license::{ActivationStatus, LicenseModel}},
};
use std::path::Path;

pub struct MacOsActivationChecker;

impl ActivationChecker for MacOsActivationChecker {
    fn check(&self, entry: &AppEntry, license_model: &LicenseModel) -> ActivationStatus {
        match license_model {
            LicenseModel::Free | LicenseModel::OpenSource | LicenseModel::Freemium => {
                ActivationStatus::NotApplicable
            }

            LicenseModel::Paid => {
                // App Store receipt — Apple has verified the purchase.
                if entry.install_path.join("Contents/_MASReceipt/receipt").exists() {
                    return ActivationStatus::AppStoreVerified;
                }

                if has_license_evidence(entry) {
                    return ActivationStatus::SelfLicensed;
                }

                ActivationStatus::Unactivated
            }

            LicenseModel::Unknown => ActivationStatus::Unknown,
        }
    }
}

fn has_license_evidence(entry: &AppEntry) -> bool {
    let Some(home) = dirs::home_dir() else { return false };

    let name_slug = entry.name.replace(' ', "");
    let bundle_id = entry.bundle_id.as_deref().unwrap_or("");

    let app_support = home.join("Library/Application Support");
    for dir in [app_support.join(&name_slug), app_support.join(bundle_id)] {
        if dir.exists() && dir_has_license_file(&dir) {
            return true;
        }
    }

    // Some apps embed a license file inside their bundle's Resources folder.
    let resources = entry.install_path.join("Contents/Resources");
    if resources.exists() && dir_has_license_file(&resources) {
        return true;
    }

    false
}

/// Returns true when `dir` directly contains a file whose name is a clear
/// license or activation artifact. Intentionally narrow to avoid false positives.
fn dir_has_license_file(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else { return false };
    for entry in entries.flatten() {
        let fname = entry.file_name();
        let n = fname.to_string_lossy().to_lowercase();
        if n.contains("license")
            || n.contains("activation")
            || n.contains("registration")
            || n.ends_with(".lic")
            || n.ends_with(".license")
            || n.ends_with(".key")
        {
            return true;
        }
    }
    false
}
