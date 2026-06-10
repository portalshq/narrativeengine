use thiserror::Error;

pub type Result<T> = std::result::Result<T, NarrativeError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NarrativeError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("serialization failed: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for NarrativeError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value.to_string())
    }
}
