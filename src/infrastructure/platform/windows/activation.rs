use crate::{
    application::ports::ActivationChecker,
    domain::{app_entry::AppEntry, license::{ActivationStatus, LicenseModel}},
};
use std::path::{Path, PathBuf};

pub struct WindowsActivationChecker;

impl ActivationChecker for WindowsActivationChecker {
    fn check(&self, entry: &AppEntry, license_model: &LicenseModel) -> ActivationStatus {
        match license_model {
            LicenseModel::Free | LicenseModel::OpenSource | LicenseModel::Freemium => {
                ActivationStatus::NotApplicable
            }

            LicenseModel::Paid => {
                // Apps in %ProgramFiles%\WindowsApps were installed via the Microsoft Store.
                let path_lower = entry.install_path.to_string_lossy().to_ascii_lowercase();
                if path_lower.contains(r"\windowsapps\") {
                    return ActivationStatus::AppStoreVerified;
                }

                let app_slug = entry.name.replace(' ', "");
                for dir in license_search_dirs(&app_slug) {
                    if dir.exists() && has_license_file(&dir) {
                        return ActivationStatus::SelfLicensed;
                    }
                }

                ActivationStatus::Unactivated
            }

            LicenseModel::Unknown => ActivationStatus::Unknown,
        }
    }
}

fn license_search_dirs(app_slug: &str) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(d) = dirs::data_dir() {
        dirs.push(d.join(app_slug));
    }
    if let Some(d) = dirs::data_local_dir() {
        dirs.push(d.join(app_slug));
    }
    if let Ok(pd) = std::env::var("PROGRAMDATA") {
        dirs.push(PathBuf::from(pd).join(app_slug));
    }
    dirs
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
