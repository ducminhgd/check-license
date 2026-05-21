use std::collections::HashMap;
use std::process::Command;

use crate::application::ports::PackageLicenseProvider;

/// Reads license data from Homebrew for installed formulae.
/// Casks are indexed by token/name but Homebrew does not expose a license field for them.
pub struct HomebrewLicenseProvider {
    index: HashMap<String, String>,
}

impl HomebrewLicenseProvider {
    pub fn load() -> Self {
        Self { index: build_index() }
    }
}

impl PackageLicenseProvider for HomebrewLicenseProvider {
    fn lookup_spdx(&self, name: &str, _bundle_id: Option<&str>) -> Option<String> {
        self.index.get(&name.to_lowercase()).cloned()
    }
}

fn build_index() -> HashMap<String, String> {
    let mut map = HashMap::new();

    let Ok(output) = Command::new("brew")
        .args(["info", "--json=v2", "--installed"])
        .output()
    else {
        return map;
    };

    let Ok(text) = std::str::from_utf8(&output.stdout) else { return map };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else { return map };

    if let Some(formulae) = json.get("formulae").and_then(|v| v.as_array()) {
        for formula in formulae {
            let name = formula.get("name").and_then(|v| v.as_str());
            let license = formula.get("license").and_then(|v| v.as_str());
            let (Some(name), Some(license)) = (name, license) else { continue };

            map.insert(name.to_lowercase(), license.to_string());

            if let Some(aliases) = formula.get("aliases").and_then(|v| v.as_array()) {
                for alias in aliases.iter().filter_map(|v| v.as_str()) {
                    map.insert(alias.to_lowercase(), license.to_string());
                }
            }
        }
    }

    map
}
