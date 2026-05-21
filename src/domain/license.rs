use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum LicenseModel {
    Free,
    Freemium,
    Paid,
    OpenSource,
    Unknown,
}

impl fmt::Display for LicenseModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LicenseModel::Free => write!(f, "Free"),
            LicenseModel::Freemium => write!(f, "Freemium"),
            LicenseModel::Paid => write!(f, "Paid"),
            LicenseModel::OpenSource => write!(f, "Open Source"),
            LicenseModel::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivationStatus {
    AppStoreVerified,
    SelfLicensed,
    Unactivated,
    NotApplicable,
    Unknown,
}

impl fmt::Display for ActivationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActivationStatus::AppStoreVerified => write!(f, "App Store"),
            ActivationStatus::SelfLicensed => write!(f, "Licensed"),
            ActivationStatus::Unactivated => write!(f, "Unactivated"),
            ActivationStatus::NotApplicable => write!(f, "N/A"),
            ActivationStatus::Unknown => write!(f, "Unknown"),
        }
    }
}
