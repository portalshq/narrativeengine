//! Entity types in the NAP protocol.
//!
//! Entity types are fully dynamic and user-defined. Each repository can have
//! any set of entity types, determined by the directory structure. A directory
//! containing a `.entity-type` marker file is treated as a valid entity type.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The kind of narrative entity a manifest describes.
///
/// Entity types are dynamic strings — any non-empty, filesystem-safe string is valid.
/// The type name corresponds directly to the directory name in the repository.
/// For example, `character` maps to `character/`, `pokemon` maps to `pokemon/`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityType(String);

/// Error returned when constructing an [`EntityType`] from an invalid string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidEntityType(pub String);

impl fmt::Display for InvalidEntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid entity type '{}': must be non-empty and contain only \
             alphanumeric characters, underscores, or hyphens (no path separators, \
             dots, or whitespace)",
            self.0
        )
    }
}

impl std::error::Error for InvalidEntityType {}

impl EntityType {
    /// Create an entity type from a string.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidEntityType`] if the string is empty, contains whitespace,
    /// path separators (`/`, `\`), dots (`.`), or other filesystem-unsafe characters.
    pub fn try_new(s: impl Into<String>) -> Result<Self, InvalidEntityType> {
        let s = s.into();
        Self::validate(&s)?;
        Ok(Self(s))
    }

    /// Create an entity type from a string without validation.
    ///
    /// # Safety
    ///
    /// This bypasses validation. Prefer [`EntityType::try_new`] for user input.
    /// Only use this when the string is known-valid (e.g., from the filesystem
    /// where it was previously validated).
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Returns the directory name for this entity type in the repository.
    /// This is simply the type name itself.
    pub fn directory_name(&self) -> &str {
        &self.0
    }

    /// Returns the entity type as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validate that a string is a legal entity type name.
    fn validate(s: &str) -> Result<(), InvalidEntityType> {
        if s.is_empty() {
            return Err(InvalidEntityType(s.to_string()));
        }
        if s.contains(|c: char| c.is_ascii_whitespace()) {
            return Err(InvalidEntityType(s.to_string()));
        }
        if s.contains('/') || s.contains('\\') {
            return Err(InvalidEntityType(s.to_string()));
        }
        if s.contains('.') {
            return Err(InvalidEntityType(s.to_string()));
        }
        if s == ".." || s == "." || s.starts_with("../") || s.starts_with("..\\") {
            return Err(InvalidEntityType(s.to_string()));
        }
        Ok(())
    }
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for EntityType {
    type Err = InvalidEntityType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s)?;
        Ok(Self(s.to_string()))
    }
}

impl AsRef<str> for EntityType {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for EntityType {
    fn from(s: String) -> Self {
        Self::try_new(s).unwrap_or_else(|_| panic!("invalid entity type"))
    }
}

impl From<&str> for EntityType {
    fn from(s: &str) -> Self {
        Self::try_new(s).unwrap_or_else(|_| panic!("invalid entity type"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_roundtrip() {
        let et = EntityType::try_new("character").unwrap();
        let s = et.to_string();
        let parsed: EntityType = s.parse().unwrap();
        assert_eq!(et, parsed);
    }

    #[test]
    fn test_entity_type_directory_name() {
        assert_eq!(
            EntityType::try_new("character").unwrap().directory_name(),
            "character"
        );
        assert_eq!(
            EntityType::try_new("pokemon").unwrap().directory_name(),
            "pokemon"
        );
        assert_eq!(
            EntityType::try_new("scientific_paper")
                .unwrap()
                .directory_name(),
            "scientific_paper"
        );
    }

    #[test]
    fn test_entity_type_from_string() {
        let et: EntityType = "custom_type".parse().unwrap();
        assert_eq!(et.as_str(), "custom_type");
    }

    #[test]
    fn test_entity_type_serialization() {
        let et = EntityType::try_new("character").unwrap();
        let yaml = serde_yaml::to_string(&et).unwrap();
        assert_eq!(yaml.trim(), "character");
    }

    #[test]
    fn test_entity_type_deserialization() {
        let et: EntityType = serde_yaml::from_str("character").unwrap();
        assert_eq!(et.as_str(), "character");
    }

    #[test]
    fn test_entity_type_rejects_empty() {
        assert!(EntityType::try_new("").is_err());
    }

    #[test]
    fn test_entity_type_rejects_path_traversal() {
        assert!(EntityType::try_new("../etc").is_err());
        assert!(EntityType::try_new("a/b").is_err());
        assert!(EntityType::try_new("a\\b").is_err());
    }

    #[test]
    fn test_entity_type_rejects_whitespace() {
        assert!(EntityType::try_new("has space").is_err());
        assert!(EntityType::try_new("has\ttab").is_err());
        assert!(EntityType::try_new("has\nnewline").is_err());
    }

    #[test]
    fn test_entity_type_rejects_dots() {
        assert!(EntityType::try_new("a.b").is_err());
        assert!(EntityType::try_new(".hidden").is_err());
    }

    #[test]
    fn test_entity_type_accepts_valid() {
        assert!(EntityType::try_new("character").is_ok());
        assert!(EntityType::try_new("Pokemon").is_ok());
        assert!(EntityType::try_new("my-type").is_ok());
        assert!(EntityType::try_new("sci_paper").is_ok());
        assert!(EntityType::try_new("T").is_ok());
    }

    #[test]
    fn test_from_str_rejects_invalid() {
        assert!("".parse::<EntityType>().is_err());
        assert!("a/b".parse::<EntityType>().is_err());
        assert!("a b".parse::<EntityType>().is_err());
    }
}
