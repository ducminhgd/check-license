use crate::{
    application::ports::AppScanner,
    domain::{app_entry::AppEntry, error::ScanError},
};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

pub struct LinuxAppScanner;

impl AppScanner for LinuxAppScanner {
    fn scan(&self) -> Result<Vec<AppEntry>, ScanError> {
        let mut entries = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        for dir in desktop_dirs() {
            scan_dir(&dir, &mut entries, &mut seen)?;
        }

        Ok(entries)
    }
}

/// Standard XDG desktop entry directories, ordered so that user-level
/// installations take priority over system-level ones.
fn desktop_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    if let Some(data_local) = dirs::data_local_dir() {
        dirs.push(data_local.join("flatpak/exports/share/applications"));
        dirs.push(data_local.join("applications"));
    }

    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    dirs.push(PathBuf::from("/var/lib/snapd/desktop/applications"));
    dirs.push(PathBuf::from("/usr/share/applications"));

    dirs.into_iter().filter(|p| p.exists()).collect()
}

fn scan_dir(
    dir: &Path,
    entries: &mut Vec<AppEntry>,
    seen: &mut HashSet<String>,
) -> Result<(), ScanError> {
    let read_dir = fs::read_dir(dir).map_err(|e| ScanError::DirectoryRead {
        path: dir.display().to_string(),
        source: e,
    })?;

    for dir_entry in read_dir.flatten() {
        let path = dir_entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
            continue;
        }
        if let Some(entry) = parse_desktop_file(&path) {
            // Deduplicate by lowercase name; user dirs are scanned first so
            // a user-level install shadows the system-level one.
            if seen.insert(entry.name.to_lowercase()) {
                entries.push(entry);
            }
        }
    }
    Ok(())
}

fn parse_desktop_file(path: &Path) -> Option<AppEntry> {
    let content = fs::read_to_string(path).ok()?;

    let mut in_desktop_entry = false;
    let mut name: Option<String> = None;
    let mut app_type: Option<String> = None;
    let mut no_display = false;
    let mut hidden = false;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }

        if !in_desktop_entry || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim();
            match key.trim() {
                // Only capture the unlocalized Name (no locale suffix like Name[fr]).
                "Name" if name.is_none() => name = Some(value.to_string()),
                "Type" => app_type = Some(value.to_string()),
                "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
                "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
                _ => {}
            }
        }
    }

    // Skip non-application entries and entries hidden from application menus.
    if app_type.as_deref() != Some("Application") || no_display || hidden {
        return None;
    }

    let name = name?;

    // Use the desktop file stem as a bundle-id-like identifier.  For Flatpak
    // apps this is the full reverse-DNS app ID (e.g. "org.mozilla.Firefox"),
    // which gives us a stable, distro-independent key.
    let bundle_id = path.file_stem().and_then(|s| s.to_str()).map(str::to_string);

    Some(AppEntry {
        name,
        bundle_id,
        version: None,
        install_path: path.to_path_buf(),
    })
}
