use crate::{
    application::ports::{OnlineLicenseInfo, OnlineLicenseProvider},
    domain::app_entry::AppEntry,
};

/// Windows online provider. The Microsoft Store does not expose a public REST API
/// for license lookups by bundle ID, so this is currently a stub. The knowledge base
/// covers common Windows apps; contributions welcome.
pub struct WindowsOnlineLicenseProvider;

impl OnlineLicenseProvider for WindowsOnlineLicenseProvider {
    fn lookup(&self, _entry: &AppEntry) -> Option<OnlineLicenseInfo> {
        None
    }
}
