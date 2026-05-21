use crate::{application::ports::{OnlineLicenseInfo, OnlineLicenseProvider}, domain::app_entry::AppEntry};

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

/// No-op provider used when --online is not set.
pub struct NullOnlineLicenseProvider;

impl OnlineLicenseProvider for NullOnlineLicenseProvider {
    fn lookup(&self, _entry: &AppEntry) -> Option<OnlineLicenseInfo> {
        None
    }
}
