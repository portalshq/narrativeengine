//! SDL — Schema Definition Language for merge behavior.
//!
//! This module defines the YAML-based schema format that controls how
//! each property is merged.  Every property MUST define a type and a
//! merge strategy.  No inferred types, no fallback heuristics.
//!
//! # SDL is schema + merge-strategy metadata only.
//!
//! Protocol invariants (missing ≠ null, normalize before merge, etc.)
//! live in `merge-semantics-v2.md` and are hardcoded in the engine.
//! They do NOT appear in SDL.
//!
//! # SDL Example
//!
//! ```yaml
//! schema:
//!   version: "1.0"
//!   required:
//!     - id
//!   properties:
//!     name:
//!       type: string
//!       merge:
//!         type: replace
//!     tags:
//!       type: array
//!       merge:
//!         type: ordered_unique
//!         identity:
//!           mode: primitive_value
//!     characters:
//!       type: array
//!       merge:
//!         type: ordered_unique
//!         identity:
//!           mode: key
//!           key: id
//!     appears_in:
//!       type: array
//!       merge:
//!         type: edge_list
//!         source_key: character_id
//!         target_key: scene_id
//!         identity:
//!           mode: key
//!           key: id
//! ```

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ── Top-level SDL Document ─────────────────────────────────────────────

/// A parsed SDL document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdlDocument {
    pub schema: SdlSchemaBody,
}

/// The body of an SDL document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdlSchemaBody {
    pub version: String,

    /// Property paths that MUST be present in every manifest.
    #[serde(default)]
    pub required: Vec<String>,

    /// Property definitions keyed by canonical path (e.g. `"name"`,
    /// `"properties.homeworld"`).
    pub properties: BTreeMap<String, PropertyDef>,
}

// ── Property Definition ────────────────────────────────────────────────

/// The definition of a single property in the SDL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDef {
    /// The property type (string, number, boolean, object, array).
    #[serde(rename = "type")]
    pub type_: PropertyType,

    /// The merge strategy for this property.
    pub merge: MergeStrategyDef,
}

// ── Property Type ──────────────────────────────────────────────────────

/// Allowed property types in SDL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyType {
    String,
    Number,
    Boolean,
    Object,
    Array,
}

// ── Merge Strategy Definition ─────────────────────────────────────────

/// The full merge strategy definition for a property.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeStrategyDef {
    /// The strategy type.
    #[serde(rename = "type")]
    pub strategy_type: MergeStrategyType,

    /// Identity rules (required for ordered_unique, set_union, edge_list).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<IdentityRule>,

    /// Source key for edge_list strategies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_key: Option<String>,

    /// Target key for edge_list strategies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_key: Option<String>,
}

/// The kind of merge strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategyType {
    /// Simple value overwrite per conflict matrix.
    Replace,
    /// Recursive object merge.  Only valid for `object` properties.
    DeepMerge,
    /// Whole value treated as atomic unit — any divergent change = conflict.
    Atomic,
    /// Ordered list with identity deduplication.  Preserves base order.
    OrderedUnique,
    /// Unordered set with identity deduplication.
    SetUnion,
    /// Graph edge list with source, target, and identity.
    EdgeList,
}

// ── Identity Rule ──────────────────────────────────────────────────────

/// Rules for determining identity in array elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum IdentityRule {
    /// Identity is the primitive value itself (for arrays of strings/numbers).
    PrimitiveValue,
    /// Identity is a named key within each object element.
    Key { key: String },
}

// ── Parsing ────────────────────────────────────────────────────────────

impl SdlDocument {
    /// Parse an SDL document from a YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Self, SdlError> {
        serde_yaml::from_str(yaml).map_err(|e| SdlError::ParseError {
            message: e.to_string(),
        })
    }

    /// Serialize this SDL document to YAML.
    pub fn to_yaml(&self) -> Result<String, SdlError> {
        serde_yaml::to_string(self).map_err(|e| SdlError::SerializeError {
            message: e.to_string(),
        })
    }

    /// Look up the property definition for a given canonical path.
    ///
    /// Returns `None` if the path is not explicitly defined in the schema.
    /// (The engine should reject schema-less properties — this is a
    /// validation concern, not a fallback.)
    pub fn property_def(&self, path: &str) -> Option<&PropertyDef> {
        self.schema.properties.get(path)
    }

    /// Returns the set of all defined property paths.
    pub fn property_paths(&self) -> impl Iterator<Item = &String> {
        self.schema.properties.keys()
    }
}

// ── Error Type ─────────────────────────────────────────────────────────

/// Errors that can occur during SDL parsing or serialization.
#[derive(Debug, thiserror::Error)]
pub enum SdlError {
    #[error("SDL parse error: {message}")]
    ParseError { message: String },

    #[error("SDL serialize error: {message}")]
    SerializeError { message: String },
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_sdl_yaml() -> &'static str {
        r#"
schema:
  version: "1.0"
  required:
    - id
  properties:
    name:
      type: string
      merge:
        type: replace
    tags:
      type: array
      merge:
        type: ordered_unique
        identity:
          mode: primitive_value
    characters:
      type: array
      merge:
        type: ordered_unique
        identity:
          mode: key
          key: id
    appears_in:
      type: array
      merge:
        type: edge_list
        source_key: character_id
        target_key: scene_id
        identity:
          mode: key
          key: id
    version:
      type: number
      merge:
        type: atomic
    metadata:
      type: object
      merge:
        type: deep_merge
    tags_set:
      type: array
      merge:
        type: set_union
        identity:
          mode: primitive_value
"#
    }

    #[test]
    fn test_parse_valid_sdl() {
        let doc = SdlDocument::from_yaml(valid_sdl_yaml()).unwrap();
        assert_eq!(doc.schema.version, "1.0");
        assert_eq!(doc.schema.required, vec!["id"]);
        assert_eq!(doc.schema.properties.len(), 7);
    }

    #[test]
    fn test_parse_replace_strategy() {
        let doc = SdlDocument::from_yaml(valid_sdl_yaml()).unwrap();
        let def = doc.property_def("name").unwrap();
        assert!(matches!(def.type_, PropertyType::String));
        assert!(matches!(
            def.merge.strategy_type,
            MergeStrategyType::Replace
        ));
    }

    #[test]
    fn test_parse_ordered_unique_primitive() {
        let doc = SdlDocument::from_yaml(valid_sdl_yaml()).unwrap();
        let def = doc.property_def("tags").unwrap();
        assert!(matches!(def.type_, PropertyType::Array));
        assert!(matches!(
            def.merge.strategy_type,
            MergeStrategyType::OrderedUnique
        ));
        let identity = def.merge.identity.as_ref().unwrap();
        assert!(matches!(identity, IdentityRule::PrimitiveValue));
    }

    #[test]
    fn test_parse_ordered_unique_key() {
        let doc = SdlDocument::from_yaml(valid_sdl_yaml()).unwrap();
        let def = doc.property_def("characters").unwrap();
        assert!(matches!(def.type_, PropertyType::Array));
        let identity = def.merge.identity.as_ref().unwrap();
        match identity {
            IdentityRule::Key { key } => assert_eq!(key, "id"),
            _ => panic!("expected Key identity"),
        }
    }

    #[test]
    fn test_parse_edge_list() {
        let doc = SdlDocument::from_yaml(valid_sdl_yaml()).unwrap();
        let def = doc.property_def("appears_in").unwrap();
        assert!(matches!(
            def.merge.strategy_type,
            MergeStrategyType::EdgeList
        ));
        assert_eq!(def.merge.source_key.as_deref(), Some("character_id"));
        assert_eq!(def.merge.target_key.as_deref(), Some("scene_id"));
    }

    #[test]
    fn test_parse_atomic() {
        let doc = SdlDocument::from_yaml(valid_sdl_yaml()).unwrap();
        let def = doc.property_def("version").unwrap();
        assert!(matches!(def.type_, PropertyType::Number));
        assert!(matches!(def.merge.strategy_type, MergeStrategyType::Atomic));
    }

    #[test]
    fn test_parse_deep_merge() {
        let doc = SdlDocument::from_yaml(valid_sdl_yaml()).unwrap();
        let def = doc.property_def("metadata").unwrap();
        assert!(matches!(def.type_, PropertyType::Object));
        assert!(matches!(
            def.merge.strategy_type,
            MergeStrategyType::DeepMerge
        ));
    }

    #[test]
    fn test_parse_set_union() {
        let doc = SdlDocument::from_yaml(valid_sdl_yaml()).unwrap();
        let def = doc.property_def("tags_set").unwrap();
        assert!(matches!(
            def.merge.strategy_type,
            MergeStrategyType::SetUnion
        ));
    }

    #[test]
    fn test_undefined_property_returns_none() {
        let doc = SdlDocument::from_yaml(valid_sdl_yaml()).unwrap();
        assert!(doc.property_def("nonexistent").is_none());
    }

    #[test]
    fn test_invalid_yaml_returns_error() {
        let result = SdlDocument::from_yaml("not: valid: yaml: [[[");
        assert!(result.is_err());
    }

    #[test]
    fn test_sdl_yaml_roundtrip() {
        let doc = SdlDocument::from_yaml(valid_sdl_yaml()).unwrap();
        let yaml = doc.to_yaml().unwrap();
        let parsed = SdlDocument::from_yaml(&yaml).unwrap();
        assert_eq!(parsed.schema.properties.len(), doc.schema.properties.len());
    }

    #[test]
    fn test_missing_merge_strategy_is_parse_error() {
        // Missing `merge` field causes a serde_yaml parse error because
        // MergeStrategyDef has no default — every property MUST declare
        // a merge strategy.
        let yaml = r#"
schema:
  version: "1.0"
  required: []
  properties:
    bad_field:
      type: string
"#;
        let result = SdlDocument::from_yaml(yaml);
        assert!(
            result.is_err(),
            "expected parse error for missing merge strategy"
        );
    }
}
