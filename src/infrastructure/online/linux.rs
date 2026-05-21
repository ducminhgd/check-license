use std::time::Duration;

use crate::{
    application::ports::{OnlineLicenseInfo, OnlineLicenseProvider},
    domain::{
        app_entry::AppEntry,
        license::{is_oss_spdx, LicenseModel},
    },
};

pub struct LinuxOnlineLicenseProvider;

impl OnlineLicenseProvider for LinuxOnlineLicenseProvider {
    fn lookup(&self, entry: &AppEntry) -> Option<OnlineLicenseInfo> {
        let bundle_id = entry.bundle_id.as_deref()?;

        if let Some(info) = lookup_flathub(bundle_id) {
            return Some(info);
        }

        lookup_snapcraft(&entry.name)
    }
}

fn lookup_flathub(bundle_id: &str) -> Option<OnlineLicenseInfo> {
    let url = format!("https://flathub.org/api/v2/appstream/{}", bundle_id);

    let resp: serde_json::Value = ureq::get(&url)
        .timeout(Duration::from_secs(10))
        .call()
        .ok()?
        .into_json()
        .ok()?;

    let license = resp.get("project_license").and_then(|v| v.as_str())?;

    let (license_model, work_allowed) = if is_oss_spdx(license) {
        (LicenseModel::OpenSource, true)
    } else {
        (LicenseModel::Free, true)
    };

    Some(OnlineLicenseInfo {
        license_model,
        work_allowed,
        spdx: Some(license.to_string()),
        source: format!("Flathub ({})", license),
    })
}

fn lookup_snapcraft(name: &str) -> Option<OnlineLicenseInfo> {
    let snap_name = name.to_lowercase().replace(' ', "-");
    let url = format!("https://api.snapcraft.io/v2/snaps/info/{}", snap_name);

    let resp: serde_json::Value = ureq::get(&url)
        .set("Snap-Device-Series", "16")
        .timeout(Duration::from_secs(10))
        .call()
        .ok()?
        .into_json()
        .ok()?;

    let license = resp
        .get("snap")
        .and_then(|s| s.get("license"))
        .and_then(|v| v.as_str())?;

    if license == "unset" || license.is_empty() {
        return None;
    }

    let (license_model, work_allowed) = if is_oss_spdx(license) {
        (LicenseModel::OpenSource, true)
    } else {
        (LicenseModel::Free, true)
    };

    Some(OnlineLicenseInfo {
        license_model,
        work_allowed,
        spdx: Some(license.to_string()),
        source: format!("Snapcraft ({})", license),
    })
}
