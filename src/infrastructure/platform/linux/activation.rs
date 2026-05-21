use crate::{
    application::ports::{ActivationChecker, AppRecord},
    domain::{
        app_entry::AppEntry,
        license::{ActivationStatus, LicenseModel},
    },
};

pub struct LinuxActivationChecker;

impl ActivationChecker for LinuxActivationChecker {
    fn check(&self, entry: &AppEntry, record: &AppRecord) -> ActivationStatus {
        if matches!(record.license_model, LicenseModel::Free | LicenseModel::OpenSource) {
            return ActivationStatus::NotApplicable;
        }

        if record.license_model == LicenseModel::Paid {
            if let Some(home) = dirs::home_dir() {
                let app_slug = entry.name.replace(' ', "");
                let candidates = [
                    home.join(".config").join(&app_slug),
                    home.join(".local/share").join(&app_slug),
                ];
                for dir in &candidates {
                    if dir.exists() && has_license_indicator(dir) {
                        return ActivationStatus::SelfLicensed;
                    }
                }
            }
            return ActivationStatus::Unactivated;
        }

        ActivationStatus::Unknown
    }
}

fn has_license_indicator(dir: &std::path::Path) -> bool {
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
