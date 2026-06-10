//! SHA-256 content addressing for NAP resources.
//!
//! Every representation (image, voice model, mesh, etc.) is hash-addressable.
//! Manifests point at content by hash, making everything:
//! - **Immutable** — content at a hash never changes
//! - **Cacheable** — same hash = same content, globally
//! - **Deduplicated** — identical content shares one hash
//! - **Verifiable** — compare hash to verify integrity

use sha2::{Digest, Sha256};
use std::fmt;
use std::path::Path;

use crate::error::NapError;

/// A SHA-256 content hash, displayed as `sha256:<hex>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ContentHash(String);

impl ContentHash {
    /// Compute the SHA-256 hash of raw bytes.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest = hasher.finalize();
        ContentHash(format!("sha256:{}", hex::encode(digest)))
    }

    /// Compute the SHA-256 hash of a string.
    pub fn from_str_content(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }

    /// Compute the SHA-256 hash of a file's contents.
    pub fn from_file(path: &Path) -> Result<Self, NapError> {
        let data = std::fs::read(path)?;
        Ok(Self::from_bytes(&data))
    }

    /// Parse a `sha256:<hex>` string into a ContentHash.
    pub fn parse(s: &str) -> Result<Self, NapError> {
        if !s.starts_with("sha256:") {
            return Err(NapError::Other(format!(
                "content hash must start with 'sha256:', got '{s}'"
            )));
        }
        let hex_part = &s[7..];
        if hex_part.len() != 64 {
            return Err(NapError::Other(format!(
                "SHA-256 hex digest must be 64 chars, got {}",
                hex_part.len()
            )));
        }
        // Validate hex characters
        hex::decode(hex_part)
            .map_err(|e| NapError::Other(format!("invalid hex in content hash: {e}")))?;
        Ok(ContentHash(s.to_string()))
    }

    /// Returns the raw hex digest without the `sha256:` prefix.
    pub fn hex_digest(&self) -> &str {
        &self.0[7..]
    }

    /// Returns the full `sha256:<hex>` string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Verify that bytes match this hash. Returns an error on mismatch.
    pub fn verify(&self, data: &[u8]) -> Result<(), NapError> {
        let actual = Self::from_bytes(data);
        if *self != actual {
            return Err(NapError::ContentHashMismatch {
                expected: self.0.clone(),
                actual: actual.0,
            });
        }
        Ok(())
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        let hash_a = ContentHash::from_str_content("hello world");
        let hash_b = ContentHash::from_str_content("hello world");
        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn test_hash_different_content() {
        let hash_a = ContentHash::from_str_content("hello");
        let hash_b = ContentHash::from_str_content("world");
        assert_ne!(hash_a, hash_b);
    }

    #[test]
    fn test_hash_format() {
        let hash = ContentHash::from_str_content("test");
        assert!(hash.as_str().starts_with("sha256:"));
        assert_eq!(hash.hex_digest().len(), 64);
    }

    #[test]
    fn test_parse_valid_hash() {
        let hash = ContentHash::from_str_content("test");
        let parsed = ContentHash::parse(hash.as_str()).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn test_parse_invalid_prefix() {
        assert!(ContentHash::parse("md5:abc123").is_err());
    }

    #[test]
    fn test_verify_success() {
        let hash = ContentHash::from_str_content("hello");
        assert!(hash.verify(b"hello").is_ok());
    }

    #[test]
    fn test_verify_mismatch() {
        let hash = ContentHash::from_str_content("hello");
        assert!(hash.verify(b"world").is_err());
    }
}
