//! Entity types in the NAP protocol.
//!
//! Each entity type has fundamentally different behavior and properties,
//! but they share a common manifest structure. The type system enforces
//! that callers are explicit about what kind of entity they are working with.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::NapError;

/// The kind of narrative entity a manifest describes.
///
/// These are NOT interchangeable — a character has memory, relationships,
/// personality, and appearance. A scene has participants, timeline, location,
/// and events. The manifest schema adapts to each type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    /// Persistent character with identity across scenes/episodes/platforms.
    Character,
    /// Spatial location within a fictional universe.
    Location,
    /// A narrative scene — participants, timeline, events.
    Scene,
    /// A physical object/prop with materials, variants, ownership.
    Prop,
    /// The universe/world itself — rules, canon, top-level metadata.
    World,
}

impl EntityType {
    /// Returns the directory name used in the repository filesystem layout.
    /// e.g., `Character` → `"characters"`, `World` → root-level `"universe.yaml"`
    pub fn directory_name(&self) -> &'static str {
        match self {
            EntityType::Character => "characters",
            EntityType::Location => "locations",
            EntityType::Scene => "scenes",
            EntityType::Prop => "props",
            EntityType::World => "", // World manifest lives at root
        }
    }

    /// Returns all entity types that have their own subdirectory in a universe repo.
    pub fn subdirectory_types() -> &'static [EntityType] {
        &[
            EntityType::Character,
            EntityType::Location,
            EntityType::Scene,
            EntityType::Prop,
        ]
    }
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityType::Character => write!(f, "character"),
            EntityType::Location => write!(f, "location"),
            EntityType::Scene => write!(f, "scene"),
            EntityType::Prop => write!(f, "prop"),
            EntityType::World => write!(f, "world"),
        }
    }
}

impl FromStr for EntityType {
    type Err = NapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "character" | "characters" => Ok(EntityType::Character),
            "location" | "locations" => Ok(EntityType::Location),
            "scene" | "scenes" => Ok(EntityType::Scene),
            "prop" | "props" => Ok(EntityType::Prop),
            "world" | "universe" => Ok(EntityType::World),
            other => Err(NapError::UnknownEntityType(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_roundtrip() {
        for entity_type in EntityType::subdirectory_types() {
            let type_str = entity_type.to_string();
            let parsed: EntityType = type_str.parse().unwrap();
            assert_eq!(*entity_type, parsed);
        }
    }

    #[test]
    fn test_entity_type_aliases() {
        assert_eq!(
            "characters".parse::<EntityType>().unwrap(),
            EntityType::Character
        );
        assert_eq!("universe".parse::<EntityType>().unwrap(), EntityType::World);
    }

    #[test]
    fn test_unknown_entity_type() {
        assert!("spaceship".parse::<EntityType>().is_err());
    }
}
