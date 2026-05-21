use crate::domain::{
    app_entry::AppEntry,
    error::{KnowledgeBaseError, ScanError},
    license::ActivationStatus,
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
    pub license_model: crate::domain::license::LicenseModel,
    pub work_allowed: bool,
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
    fn check(&self, entry: &AppEntry, record: &AppRecord) -> ActivationStatus;
}
