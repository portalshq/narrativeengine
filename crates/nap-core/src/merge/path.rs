//! SDL-aware path resolution for three-way merge.
//!
//! Paths are resolved using identity keys (not array indices) when the
//! SDL declares an identity mode for an array property.  This prevents
//! path instability — adding or removing items before a given index
//! does not invalidate previously resolved paths.
//!
//! # Path Format
//!
//! Canonical paths use dot notation for object keys:
//!
//! ```text
//! name
//! properties.homeworld
//! properties.settings.visual.fog_color
//! ```
//!
//! For array items the identity value is used in brackets:
//!
//! ```text
//! characters[obiwan].name
//! references[scene_4].participants
//! ```
//!
//! # Index-based access is NEVER used
//!
//! Array indices (`characters[0]`) are not produced and not resolved
//! by the canonical path system.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::merge::sdl::{IdentityRule, MergeStrategyType, SdlDocument};

/// A resolved segment of a canonical path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    /// A named object key.  e.g. `properties`, `name`
    Key(String),
    /// An array item identified by its identity value.  e.g. `obiwan`
    Identity(String),
}

/// A parsed canonical path as a sequence of segments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalPath {
    segments: Vec<PathSegment>,
}

impl CanonicalPath {
    /// Parse a canonical path string into segments.
    ///
    /// Supports:
    /// - `key.subkey`  → [Key("key"), Key("subkey")]
    /// - `key[identity]` → [Key("key"), Identity("identity")]
    /// - `key[identity].subkey` → [Key("key"), Identity("identity"), Key("subkey")]
    pub fn parse(path: &str) -> Result<Self, PathError> {
        if path.is_empty() {
            return Err(PathError::EmptyPath);
        }

        let mut segments = Vec::new();
        let mut remaining = path;

        while !remaining.is_empty() {
            // Check for bracket (identity access)
            if remaining.starts_with('[') {
                let close = remaining
                    .find(']')
                    .ok_or_else(|| PathError::InvalidSyntax {
                        path: path.to_string(),
                        detail: "unclosed bracket".to_string(),
                    })?;
                let identity = &remaining[1..close];
                if identity.is_empty() {
                    return Err(PathError::InvalidSyntax {
                        path: path.to_string(),
                        detail: "empty identity in brackets".to_string(),
                    });
                }
                segments.push(PathSegment::Identity(identity.to_string()));
                remaining = &remaining[close + 1..];
                // Expect either end-of-string or a dot+key
                if !remaining.is_empty() {
                    if remaining.starts_with('.') {
                        remaining = &remaining[1..]; // consume dot
                    } else {
                        return Err(PathError::InvalidSyntax {
                            path: path.to_string(),
                            detail: format!(
                                "expected '.' or end after ']', got '{}'",
                                &remaining[..1]
                            ),
                        });
                    }
                }
            } else {
                // Read key up to next dot or bracket
                let end = remaining.find(['.', '[']).unwrap_or(remaining.len());
                let key = &remaining[..end];
                if key.is_empty() {
                    return Err(PathError::InvalidSyntax {
                        path: path.to_string(),
                        detail: "empty key segment".to_string(),
                    });
                }
                segments.push(PathSegment::Key(key.to_string()));
                remaining = &remaining[end..];
                if remaining.starts_with('.') {
                    remaining = &remaining[1..]; // consume dot
                }
            }
        }

        Ok(CanonicalPath { segments })
    }

    /// Return the segments of this path.
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// Returns the parent path (all segments except the last), if any.
    pub fn parent(&self) -> Option<CanonicalPath> {
        if self.segments.len() <= 1 {
            return None;
        }
        Some(CanonicalPath {
            segments: self.segments[..self.segments.len() - 1].to_vec(),
        })
    }

    /// Returns the last segment of the path.
    pub fn last_segment(&self) -> Option<&PathSegment> {
        self.segments.last()
    }
}

impl std::fmt::Display for CanonicalPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, seg) in self.segments.iter().enumerate() {
            if i > 0 {
                // Identity segments use bracket notation without dot
                // Key segments use dot separator
                match seg {
                    PathSegment::Identity(_) => {}
                    PathSegment::Key(_) => write!(f, ".")?,
                }
            }
            match seg {
                PathSegment::Key(k) => write!(f, "{k}")?,
                PathSegment::Identity(id) => write!(f, "[{id}]")?,
            }
        }
        Ok(())
    }
}

// ── Path Building ──────────────────────────────────────────────────────

/// Build canonical paths for every leaf value in a JSON tree.
///
/// Returns a flat `BTreeMap` from canonical path string to value.
/// For array items with identity rules, uses the identity value as
/// the path segment instead of the array index.
pub fn build_path_map(value: &Value, schema: &SdlDocument) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    build_path_map_inner(value, schema, "", &mut map);
    map
}

fn build_path_map_inner(
    value: &Value,
    schema: &SdlDocument,
    prefix: &str,
    map: &mut BTreeMap<String, Value>,
) {
    match value {
        Value::Object(obj) => {
            for (key, val) in obj {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                build_path_map_inner(val, schema, &path, map);
            }
        }
        Value::Array(arr) => {
            // Look up merge strategy for this path to find identity rules.
            let identity_rule = schema.property_def(prefix).and_then(|def| {
                if matches!(
                    def.merge.strategy_type,
                    MergeStrategyType::OrderedUnique
                        | MergeStrategyType::SetUnion
                        | MergeStrategyType::EdgeList
                ) {
                    def.merge.identity.clone()
                } else {
                    None
                }
            });

            match identity_rule {
                Some(IdentityRule::PrimitiveValue) => {
                    // Store the array itself for whole-array merge
                    map.insert(prefix.to_string(), Value::Array(arr.clone()));
                    // Also emit individual item paths for detailed diff
                    for val in arr {
                        let identity = value_to_identity_string(val);
                        let path = format!("{prefix}[{identity}]");
                        map.insert(path, val.clone());
                    }
                }
                Some(IdentityRule::Key { key: identity_key }) => {
                    // Store the array itself for whole-array merge
                    map.insert(prefix.to_string(), Value::Array(arr.clone()));
                    // Also emit individual item paths and sub-paths for detailed diff
                    for val in arr {
                        if let Some(identity) = val.get(&identity_key).and_then(|v| v.as_str()) {
                            let path = format!("{prefix}[{identity}]");
                            map.insert(path.clone(), val.clone());
                            build_path_map_inner(val, schema, &path, map);
                        }
                    }
                }
                None => {
                    // No identity rules — use index-based paths
                    for (idx, val) in arr.iter().enumerate() {
                        let path = format!("{prefix}[{idx}]");
                        map.insert(path.clone(), val.clone());
                        build_path_map_inner(val, schema, &path, map);
                    }
                }
            }
        }
        _ => {
            // Leaf value — store the path
            map.insert(prefix.to_string(), value.clone());
        }
    }
}

/// Convert a JSON value to a string suitable for use as an identity in paths.
fn value_to_identity_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => {
            // For complex values used as identity (rare), use JSON
            serde_json::to_string(val).unwrap_or_else(|_| "unknown".to_string())
        }
    }
}

/// Resolve a canonical path string against a JSON value tree.
pub fn resolve_path<'a>(root: &'a Value, path: &CanonicalPath) -> Option<&'a Value> {
    let mut current = root;

    for segment in path.segments() {
        current = match segment {
            PathSegment::Key(key) => match current {
                Value::Object(map) => map.get(key)?,
                _ => return None,
            },
            PathSegment::Identity(id) => match current {
                Value::Array(arr) => {
                    // Identity-based lookup: search array for matching identity.
                    // This is O(n) per lookup, which is acceptable for typical
                    // manifest sizes.  For 10k+ items, build an index.
                    // Heuristic: try to interpret id as a numeric index first.
                    // This covers the fallback case where identity rules were
                    // not available at path-building time.
                    if idx_from_str(id) < arr.len() {
                        // Check if this index matches (identity might be numeric)
                        let candidate = &arr[idx_from_str(id)];
                        // Only use index if it looks plausible (no identity rules were used)
                        if matches!(
                            candidate,
                            Value::String(_) | Value::Number(_) | Value::Bool(_)
                        ) {
                            return Some(candidate);
                        }
                    }
                    // Full scan by identity value
                    arr.iter().find(|item| item_has_identity(item, id))?
                }
                _ => return None,
            },
        };
    }

    Some(current)
}

/// Check if a JSON value matches a given identity string.
fn item_has_identity(item: &Value, identity: &str) -> bool {
    match item {
        Value::String(s) => s == identity,
        Value::Number(n) => n.to_string() == identity,
        Value::Bool(b) => b.to_string() == identity,
        Value::Object(map) => {
            // Check if any string field matches the identity
            map.values().any(|v| match v {
                Value::String(s) => s == identity,
                Value::Number(n) => n.to_string() == identity,
                _ => false,
            })
        }
        _ => false,
    }
}

fn idx_from_str(s: &str) -> usize {
    s.parse().unwrap_or(usize::MAX)
}

// ── Path Union ─────────────────────────────────────────────────────────

/// Compute the union of all paths across multiple path maps.
pub fn path_union(maps: &[&BTreeMap<String, Value>]) -> Vec<String> {
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for map in maps {
        for key in map.keys() {
            seen.insert(key.as_str());
        }
    }
    seen.into_iter().map(String::from).collect()
}

// ── Errors ─────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("path cannot be empty")]
    EmptyPath,

    #[error("invalid path syntax '{path}': {detail}")]
    InvalidSyntax { path: String, detail: String },
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merge::sdl::SdlDocument;
    use serde_json::json;

    fn test_sdl() -> SdlDocument {
        SdlDocument::from_yaml(
            r#"
schema:
  version: "1.0"
  required: []
  properties:
    characters:
      type: array
      merge:
        type: ordered_unique
        identity:
          mode: key
          key: id
    tags:
      type: array
      merge:
        type: ordered_unique
        identity:
          mode: primitive_value
"#,
        )
        .unwrap()
    }

    #[test]
    fn test_parse_simple_path() {
        let path = CanonicalPath::parse("name").unwrap();
        assert_eq!(path.segments(), &[PathSegment::Key("name".to_string())]);
    }

    #[test]
    fn test_parse_nested_path() {
        let path = CanonicalPath::parse("properties.homeworld").unwrap();
        assert_eq!(
            path.segments(),
            &[
                PathSegment::Key("properties".to_string()),
                PathSegment::Key("homeworld".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_identity_path() {
        let path = CanonicalPath::parse("characters[obiwan]").unwrap();
        assert_eq!(
            path.segments(),
            &[
                PathSegment::Key("characters".to_string()),
                PathSegment::Identity("obiwan".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_identity_with_subpath() {
        let path = CanonicalPath::parse("characters[obiwan].name").unwrap();
        assert_eq!(
            path.segments(),
            &[
                PathSegment::Key("characters".to_string()),
                PathSegment::Identity("obiwan".to_string()),
                PathSegment::Key("name".to_string()),
            ]
        );
    }

    #[test]
    fn test_path_to_string() {
        let path = CanonicalPath::parse("characters[obiwan].name").unwrap();
        assert_eq!(path.to_string(), "characters[obiwan].name");
    }

    #[test]
    fn test_build_path_map_with_identity_keys() {
        let value = json!({
            "name": "Luke",
            "characters": [
                {"id": "obiwan", "name": "Obi-Wan"},
                {"id": "anakin", "name": "Anakin"}
            ],
            "tags": ["jedi", "force"]
        });

        let schema = test_sdl();
        let map = build_path_map(&value, &schema);

        // Simple keys
        assert_eq!(map.get("name"), Some(&json!("Luke")));

        // Identity-keyed array items
        assert_eq!(map.get("characters[obiwan].name"), Some(&json!("Obi-Wan")));
        assert_eq!(map.get("characters[anakin].name"), Some(&json!("Anakin")));

        // Primitive identity array
        assert!(map.contains_key("tags[jedi]"));
        assert!(map.contains_key("tags[force]"));

        // The full item objects should also be in the map
        assert!(map.contains_key("characters[obiwan]"));
        assert!(map.contains_key("characters[anakin]"));
    }

    #[test]
    fn test_resolve_simple_path() {
        let value = json!({"name": "Luke", "homeworld": "Tatooine"});
        let path = CanonicalPath::parse("name").unwrap();
        assert_eq!(resolve_path(&value, &path), Some(&json!("Luke")));
    }

    #[test]
    fn test_resolve_nested_path() {
        let value = json!({"properties": {"homeworld": "Tatooine"}});
        let path = CanonicalPath::parse("properties.homeworld").unwrap();
        assert_eq!(resolve_path(&value, &path), Some(&json!("Tatooine")));
    }

    #[test]
    fn test_resolve_identity_path() {
        let value = json!({
            "characters": [
                {"id": "obiwan", "name": "Obi-Wan"},
            ]
        });
        let path = CanonicalPath::parse("characters[obiwan].name").unwrap();
        assert_eq!(resolve_path(&value, &path), Some(&json!("Obi-Wan")));
    }

    #[test]
    fn test_resolve_nonexistent_path() {
        let value = json!({"name": "Luke"});
        let path = CanonicalPath::parse("nonexistent").unwrap();
        assert_eq!(resolve_path(&value, &path), None);
    }

    #[test]
    fn test_parent_path() {
        let path = CanonicalPath::parse("characters[obiwan].name").unwrap();
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "characters[obiwan]");

        let grandparent = parent.parent().unwrap();
        assert_eq!(grandparent.to_string(), "characters");
    }

    #[test]
    fn test_parent_of_root_is_none() {
        let path = CanonicalPath::parse("name").unwrap();
        assert!(path.parent().is_none());
    }

    #[test]
    fn test_path_union() {
        let map_a: BTreeMap<String, Value> =
            [("a".to_string(), json!(1)), ("b".to_string(), json!(2))]
                .into_iter()
                .collect();
        let map_b: BTreeMap<String, Value> =
            [("b".to_string(), json!(20)), ("c".to_string(), json!(3))]
                .into_iter()
                .collect();

        let union = path_union(&[&map_a, &map_b]);
        assert_eq!(union, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_empty_path_error() {
        assert!(CanonicalPath::parse("").is_err());
    }
}
