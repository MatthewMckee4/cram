use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("deck not found: {0}")]
    NotFound(String),
    #[error("directory not found: {0}")]
    DirNotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),
}
