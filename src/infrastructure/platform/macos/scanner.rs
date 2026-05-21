use crate::{
    application::ports::AppScanner,
    domain::{app_entry::AppEntry, error::ScanError},
};
use std::path::{Path, PathBuf};

pub struct MacOsAppScanner;

impl AppScanner for MacOsAppScanner {
    fn scan(&self) -> Result<Vec<AppEntry>, ScanError> {
        let mut entries = Vec::new();

        let search_dirs: Vec<PathBuf> = [
            PathBuf::from("/Applications"),
            dirs::home_dir().map(|h| h.join("Applications")).unwrap_or_default(),
        ]
        .into_iter()
        .filter(|p| p.exists())
        .collect();

        for dir in &search_dirs {
            self.scan_dir(dir, &mut entries)?;
        }

        Ok(entries)
    }
}

impl MacOsAppScanner {
    fn scan_dir(&self, dir: &Path, entries: &mut Vec<AppEntry>) -> Result<(), ScanError> {
        let read_dir = std::fs::read_dir(dir).map_err(|e| ScanError::DirectoryRead {
            path: dir.display().to_string(),
            source: e,
        })?;

        for dir_entry in read_dir.flatten() {
            let path = dir_entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("app") {
                continue;
            }
            if let Some(entry) = self.parse_app(&path) {
                entries.push(entry);
            }
        }
        Ok(())
    }

    fn parse_app(&self, app_path: &Path) -> Option<AppEntry> {
        let plist_path = app_path.join("Contents/Info.plist");
        if !plist_path.exists() {
            return None;
        }

        let val = plist::Value::from_file(&plist_path).ok()?;
        let dict = val.as_dictionary()?;

        let name = dict
            .get("CFBundleName")
            .or_else(|| dict.get("CFBundleDisplayName"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                app_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            });

        let bundle_id = dict
            .get("CFBundleIdentifier")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());

        let version = dict
            .get("CFBundleShortVersionString")
            .or_else(|| dict.get("CFBundleVersion"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());

        Some(AppEntry {
            name,
            bundle_id,
            version,
            install_path: app_path.to_path_buf(),
        })
    }
}
