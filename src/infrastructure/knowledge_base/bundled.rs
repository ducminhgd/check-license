use crate::{
    application::ports::{AppRecord, KnowledgeBase},
    domain::error::KnowledgeBaseError,
};
use serde::Deserialize;

static KNOWLEDGE_BASE_JSON: &str = include_str!("../../../data/known_apps.json");

#[derive(Deserialize)]
struct KnowledgeBaseFile {
    #[allow(dead_code)]
    schema_version: u32,
    apps: Vec<AppRecord>,
}

pub struct BundledKnowledgeBase {
    apps: Vec<AppRecord>,
}

impl KnowledgeBase for BundledKnowledgeBase {
    fn load() -> Result<Self, KnowledgeBaseError> {
        let file: KnowledgeBaseFile = serde_json::from_str(KNOWLEDGE_BASE_JSON)
            .map_err(|e| KnowledgeBaseError::ParseError(e.to_string()))?;
        Ok(Self { apps: file.apps })
    }

    fn lookup(&self, bundle_id: Option<&str>, name: &str) -> Option<&AppRecord> {
        // Exact bundle_id match first.
        if let Some(bid) = bundle_id {
            if let Some(record) = self.apps.iter().find(|r| {
                r.bundle_id.as_deref().map(|b| b == bid).unwrap_or(false)
            }) {
                return Some(record);
            }
        }
        // Case-insensitive name fallback.
        let name_lower = name.to_lowercase();
        self.apps.iter().find(|r| r.name.to_lowercase() == name_lower)
    }
}
