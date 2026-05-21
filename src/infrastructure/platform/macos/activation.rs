use crate::{
    application::ports::{ActivationChecker, AppRecord},
    domain::{app_entry::AppEntry, license::{ActivationStatus, LicenseModel}},
};

pub struct MacOsActivationChecker;

impl ActivationChecker for MacOsActivationChecker {
    fn check(&self, entry: &AppEntry, record: &AppRecord) -> ActivationStatus {
        // Free / Open Source apps need no activation check.
        if matches!(record.license_model, LicenseModel::Free | LicenseModel::OpenSource) {
            return ActivationStatus::NotApplicable;
        }

        // App Store receipt: presence means Apple has verified the purchase.
        let receipt_path = entry.install_path.join("Contents/_MASReceipt/receipt");
        if receipt_path.exists() {
            return ActivationStatus::AppStoreVerified;
        }

        // For paid apps not from the App Store, check for common license file indicators.
        // Many apps write a license file, activation plist, or similar under ~/Library.
        if record.license_model == LicenseModel::Paid {
            if let Some(home) = dirs::home_dir() {
                let lib = home.join("Library");
                // Heuristic: if the app has a support directory under ~/Library/Application Support
                // with a file named after the app, treat as self-licensed.
                // This is intentionally conservative — we only set SelfLicensed if there is
                // positive evidence; absence alone leads to Unactivated.
                let support_dir = lib
                    .join("Application Support")
                    .join(entry.name.replace(' ', ""));
                if support_dir.exists() && has_license_indicator(&support_dir) {
                    return ActivationStatus::SelfLicensed;
                }
            }
            return ActivationStatus::Unactivated;
        }

        ActivationStatus::Unknown
    }
}

/// Returns true when the directory contains a file whose name suggests it is a
/// license or activation artifact. Deliberately narrow — false negatives are
/// preferable to false positives that mask real cracks.
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
