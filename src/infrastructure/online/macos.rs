use std::time::Duration;

use crate::{
    application::ports::{OnlineLicenseInfo, OnlineLicenseProvider},
    domain::{app_entry::AppEntry, license::LicenseModel},
};

pub struct MacOsOnlineLicenseProvider;

impl OnlineLicenseProvider for MacOsOnlineLicenseProvider {
    fn lookup(&self, entry: &AppEntry) -> Option<OnlineLicenseInfo> {
        let bundle_id = entry.bundle_id.as_deref()?;
        lookup_itunes(bundle_id)
    }
}

/// Queries the Apple iTunes Search API for App Store metadata by bundle ID.
/// Returns None when the app is not listed on the App Store or the request fails.
fn lookup_itunes(bundle_id: &str) -> Option<OnlineLicenseInfo> {
    let url = format!(
        "https://itunes.apple.com/lookup?bundleId={}&entity=software&country=us",
        bundle_id
    );

    let resp: serde_json::Value = ureq::get(&url)
        .timeout(Duration::from_secs(10))
        .call()
        .ok()?
        .into_json()
        .ok()?;

    let result_count = resp.get("resultCount").and_then(|v| v.as_u64()).unwrap_or(0);
    if result_count == 0 {
        return None;
    }

    let app = resp.get("results")?.as_array()?.first()?;

    let price = app.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let formatted_price = app
        .get("formattedPrice")
        .and_then(|v| v.as_str())
        .unwrap_or(if price == 0.0 { "Free" } else { "Paid" });

    // work_allowed: App Store purchases grant commercial use unless the app's own
    // ToS restricts it. Default to true; the KB can override known exceptions.
    let (license_model, work_allowed) = if price == 0.0 {
        (LicenseModel::Free, true)
    } else {
        (LicenseModel::Paid, true)
    };

    Some(OnlineLicenseInfo {
        license_model,
        work_allowed,
        spdx: None, // iTunes does not expose an SPDX identifier
        source: format!("App Store ({})", formatted_price),
    })
}
