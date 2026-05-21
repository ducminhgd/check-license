use crate::{
    application::ports::AppScanner,
    domain::{app_entry::AppEntry, error::ScanError},
};
use std::{collections::HashSet, path::PathBuf};
use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    RegKey,
};

pub struct WindowsAppScanner;

impl AppScanner for WindowsAppScanner {
    fn scan(&self) -> Result<Vec<AppEntry>, ScanError> {
        let mut entries = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // 64-bit machine-wide, 32-bit machine-wide (WOW64), and per-user installs.
        let uninstall_keys = [
            hklm.open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
            hklm.open_subkey(r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
            hkcu.open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        ];

        for result in uninstall_keys {
            let Ok(key) = result else { continue };
            scan_uninstall_key(&key, &mut entries, &mut seen);
        }

        Ok(entries)
    }
}

fn scan_uninstall_key(
    uninstall_key: &RegKey,
    entries: &mut Vec<AppEntry>,
    seen: &mut HashSet<String>,
) {
    for key_name in uninstall_key.enum_keys().flatten() {
        let Ok(app_key) = uninstall_key.open_subkey(&key_name) else { continue };
        if let Some(entry) = parse_entry(&app_key, &key_name) {
            // Deduplicate by normalised name; the same app often appears under
            // both the 64-bit and WOW64 Uninstall keys.
            if seen.insert(entry.name.to_lowercase()) {
                entries.push(entry);
            }
        }
    }
}

fn parse_entry(key: &RegKey, key_name: &str) -> Option<AppEntry> {
    let raw_name: String = key.get_value("DisplayName").ok()?;
    let raw_name = raw_name.trim();
    if raw_name.is_empty() {
        return None;
    }

    // Skip Windows OS components and internal service entries.
    let system_component: u32 = key.get_value("SystemComponent").unwrap_or(0u32);
    if system_component != 0 {
        return None;
    }

    // Skip updates, hotfixes, and service packs.
    let release_type: String = key.get_value("ReleaseType").unwrap_or_default();
    if !release_type.is_empty() {
        return None;
    }

    let version: Option<String> = key
        .get_value::<String, _>("DisplayVersion")
        .ok()
        .filter(|v| !v.is_empty());

    let install_path: PathBuf = key
        .get_value::<String, _>("InstallLocation")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_default();

    // Strip architecture suffixes many installers append to the display name
    // so that "Notepad++ (64-bit x64)" matches "Notepad++" in the knowledge base.
    let name = strip_arch_suffix(raw_name);

    // Use the registry key name as a stable identifier. For MSIX/Store apps
    // this is the package family name; for MSI packages it is the product GUID.
    let bundle_id = (!key_name.is_empty()).then(|| key_name.to_string());

    Some(AppEntry { name, bundle_id, version, install_path })
}

/// Strips installer-appended architecture tags from the end of a display name.
fn strip_arch_suffix(name: &str) -> String {
    const SUFFIXES: &[&str] = &[
        " (64-bit x64)",
        " (64-bit)",
        " (x64 edition)",
        " (x64)",
        " (32-bit x86)",
        " (32-bit)",
        " (x86 edition)",
        " (x86)",
    ];
    for suffix in SUFFIXES {
        if let Some(stripped) = name.strip_suffix(suffix) {
            return stripped.trim().to_string();
        }
    }
    name.to_string()
}
