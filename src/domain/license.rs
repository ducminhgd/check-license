use serde::Deserialize;
use std::fmt;

pub fn is_oss_spdx(spdx: &str) -> bool {
    let upper = spdx.trim().to_uppercase();

    const NON_OSS: &[&str] = &[
        "PROPRIETARY", "COMMERCIAL", "FREEWARE", "SHAREWARE",
        "ALL RIGHTS RESERVED", "EULA",
    ];
    for non_oss in NON_OSS {
        if upper.contains(non_oss) {
            return false;
        }
    }

    const OSS_PREFIXES: &[&str] = &[
        "MIT", "APACHE", "GPL", "LGPL", "AGPL", "BSD", "ISC", "MPL", "CDDL",
        "EPL", "EUPL", "ZLIB", "CC0", "UNLICENSE", "0BSD", "WTFPL",
        "ARTISTIC", "AFL", "OSL", "NCSA", "CPL", "PHP", "PYTHON",
        "POSTGRESQL", "RUBY", "W3C", "ZPL", "OFL", "HPND", "CECILL", "LPPL",
        "CPAL", "RPSL", "SPL", "MS-PL", "MS-RL", "NOKIA", "NTP", "OCLC",
    ];
    OSS_PREFIXES.iter().any(|prefix| upper.contains(prefix))
}

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
