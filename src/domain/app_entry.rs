use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    pub bundle_id: Option<String>,
    pub version: Option<String>,
    pub install_path: PathBuf,
}
