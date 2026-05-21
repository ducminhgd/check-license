use crate::{
    application::ports::ActivationChecker,
    domain::{app_entry::AppEntry, license::{ActivationStatus, LicenseModel}},
};
use std::path::Path;

pub struct LinuxActivationChecker;

impl ActivationChecker for LinuxActivationChecker {
    fn check(&self, entry: &AppEntry, license_model: &LicenseModel) -> ActivationStatus {
        match license_model {
            LicenseModel::Free | LicenseModel::OpenSource | LicenseModel::Freemium => {
                ActivationStatus::NotApplicable
            }

            LicenseModel::Paid => {
                let Some(home) = dirs::home_dir() else {
                    return ActivationStatus::Unactivated;
                };

                let name_slug = entry.name.replace(' ', "");
                let bundle_id = entry.bundle_id.as_deref().unwrap_or("");

                let candidates: &[std::path::PathBuf] = &[
                    home.join(".config").join(&name_slug),
                    home.join(".config").join(bundle_id),
                    home.join(".local/share").join(&name_slug),
                    home.join(".local/share").join(bundle_id),
                ];

                for dir in candidates {
                    if dir.exists() && has_license_file(dir) {
                        return ActivationStatus::SelfLicensed;
                    }
                }

                ActivationStatus::Unactivated
            }

            LicenseModel::Unknown => ActivationStatus::Unknown,
        }
    }
}

fn has_license_file(dir: &Path) -> bool {
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
