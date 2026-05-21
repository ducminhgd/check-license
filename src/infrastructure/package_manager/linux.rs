use std::collections::HashMap;
use std::process::Command;

use crate::application::ports::PackageLicenseProvider;

/// Reads license data from the system package manager (RPM or dpkg).
pub struct LinuxLicenseProvider {
    index: HashMap<String, String>,
}

impl LinuxLicenseProvider {
    pub fn load() -> Self {
        let mut index = HashMap::new();
        load_rpm(&mut index);
        load_dpkg(&mut index);
        Self { index }
    }
}

impl PackageLicenseProvider for LinuxLicenseProvider {
    fn lookup_spdx(&self, name: &str, _bundle_id: Option<&str>) -> Option<String> {
        self.index.get(&name.to_lowercase()).cloned()
    }
}

fn load_rpm(map: &mut HashMap<String, String>) {
    let Ok(output) = Command::new("rpm")
        .args(["-qa", "--queryformat", "%{NAME}|%{LICENSE}\\n"])
        .output()
    else {
        return;
    };

    let Ok(text) = std::str::from_utf8(&output.stdout) else { return };
    for line in text.lines() {
        let mut parts = line.splitn(2, '|');
        let name = parts.next().unwrap_or("").trim();
        let license = parts.next().unwrap_or("").trim();
        if !name.is_empty() && !license.is_empty() && license != "(none)" {
            map.insert(name.to_lowercase(), license.to_string());
        }
    }
}

fn load_dpkg(map: &mut HashMap<String, String>) {
    let Ok(output) = Command::new("dpkg-query")
        .args(["-W", "-f", "${Package}\\n"])
        .output()
    else {
        return;
    };

    let Ok(text) = std::str::from_utf8(&output.stdout) else { return };
    for pkg in text.lines() {
        let pkg = pkg.trim();
        if pkg.is_empty() {
            continue;
        }
        let copyright_path =
            std::path::PathBuf::from("/usr/share/doc").join(pkg).join("copyright");
        let Ok(content) = std::fs::read_to_string(&copyright_path) else { continue };
        if let Some(license) = extract_dep5_license(&content) {
            map.insert(pkg.to_lowercase(), license);
        }
    }
}

/// Extracts the top-level `License:` field from a DEP-5 machine-readable copyright file.
fn extract_dep5_license(content: &str) -> Option<String> {
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("License:") {
            let lic = rest.trim();
            // Skip paragraph-level license blocks that start with '(' (complex expressions)
            // and empty values; take the first simple identifier.
            if !lic.is_empty() && !lic.starts_with('(') {
                return Some(lic.to_string());
            }
        }
    }
    None
}
