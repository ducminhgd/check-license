use crate::domain::{
    app_entry::AppEntry,
    error::{KnowledgeBaseError, ScanError},
    license::{ActivationStatus, LicenseModel},
};
use serde::Deserialize;

pub trait AppScanner {
    fn scan(&self) -> Result<Vec<AppEntry>, ScanError>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrackIndicators {
    #[serde(default)]
    pub hosts_entries: Vec<String>,
    #[serde(default)]
    pub known_crack_app_bundle_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppRecord {
    pub bundle_id: Option<String>,
    pub name: String,
    pub license_model: LicenseModel,
    pub work_allowed: bool,
    /// SPDX license identifier, e.g. "MIT", "GPL-2.0-only", "Proprietary".
    #[serde(default)]
    pub spdx: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub crack_indicators: Option<CrackIndicators>,
}

pub trait KnowledgeBase {
    fn load() -> Result<Self, KnowledgeBaseError>
    where
        Self: Sized;

    /// Look up by bundle_id first, fall back to case-insensitive name match.
    fn lookup(&self, bundle_id: Option<&str>, name: &str) -> Option<&AppRecord>;
}

pub trait ActivationChecker {
    /// Determine the activation status of an app given its resolved license model.
    /// Does not require a KB record — works for any scanned entry.
    fn check(&self, entry: &AppEntry, license_model: &LicenseModel) -> ActivationStatus;
}

pub trait PackageLicenseProvider {
    /// Returns the SPDX license expression for the app, or None if unknown.
    fn lookup_spdx(&self, name: &str, bundle_id: Option<&str>) -> Option<String>;
}

/// Data returned by an online license lookup.
pub struct OnlineLicenseInfo {
    pub license_model: LicenseModel,
    pub work_allowed: bool,
    /// SPDX identifier when the remote source provides one, e.g. "GPL-3.0-only".
    pub spdx: Option<String>,
    /// Human-readable note appended to the Notes column, e.g. "Flathub (GPL-3.0-only)".
    pub source: String,
}

pub trait OnlineLicenseProvider {
    /// Query an internet source for license/pricing information about the app.
    /// Returns None if the app cannot be found or the request fails.
    fn lookup(&self, entry: &AppEntry) -> Option<OnlineLicenseInfo>;
}
