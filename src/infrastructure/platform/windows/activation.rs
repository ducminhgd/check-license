use crate::{
    application::ports::{ActivationChecker, AppRecord},
    domain::{
        app_entry::AppEntry,
        license::{ActivationStatus, LicenseModel},
    },
};
use std::path::{Path, PathBuf};

pub struct WindowsActivationChecker;

impl ActivationChecker for WindowsActivationChecker {
    fn check(&self, entry: &AppEntry, record: &AppRecord) -> ActivationStatus {
        if matches!(record.license_model, LicenseModel::Free | LicenseModel::OpenSource) {
            return ActivationStatus::NotApplicable;
        }

        // Apps located under %ProgramFiles%\WindowsApps\ were installed
        // through the Microsoft Store and are purchase-verified.
        let path_lower = entry.install_path.to_string_lossy().to_ascii_lowercase();
        if path_lower.contains(r"\windowsapps\") {
            return ActivationStatus::AppStoreVerified;
        }

        if record.license_model == LicenseModel::Paid {
            let app_slug = entry.name.replace(' ', "");
            for dir in license_search_dirs(&app_slug) {
                if dir.exists() && has_license_indicator(&dir) {
                    return ActivationStatus::SelfLicensed;
                }
            }
            return ActivationStatus::Unactivated;
        }

        ActivationStatus::Unknown
    }
}

/// Candidate directories where commercial Windows apps commonly store license
/// or activation artefacts.
fn license_search_dirs(app_slug: &str) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // %APPDATA%  (C:\Users\<user>\AppData\Roaming)
    if let Some(d) = dirs::data_dir() {
        dirs.push(d.join(app_slug));
    }
    // %LOCALAPPDATA%  (C:\Users\<user>\AppData\Local)
    if let Some(d) = dirs::data_local_dir() {
        dirs.push(d.join(app_slug));
    }
    // %PROGRAMDATA%  (C:\ProgramData)
    if let Some(d) = std::env::var("PROGRAMDATA").ok().map(PathBuf::from) {
        dirs.push(d.join(app_slug));
    }

    dirs
}

fn has_license_indicator(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else { return false };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_lowercase();
        if name_str.contains("license")
            || name_str.contains("activation")
            || name_str.contains("registration")
            || name_str.ends_with(".lic")
        {
            return true;
        }
    }
    false
}
