use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("failed to read directory {path}: {source}")]
    DirectoryRead {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse app metadata at {path}: {reason}")]
    #[allow(dead_code)]
    MetadataParse { path: String, reason: String },
}

#[derive(Debug, Error)]
pub enum KnowledgeBaseError {
    #[error("failed to parse bundled knowledge base: {0}")]
    ParseError(String),
}
