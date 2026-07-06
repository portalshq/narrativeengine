//! Subtree query engine for NAP manifests.
//!
//! Supports dot-notation path traversal into YAML/JSON manifest values:
//!
//! ```text
//! "appearances.audienceVotes"  → manifest["appearances"]["audienceVotes"]
//! "representations.reference_image.hash" → the BLAKE3 hash string
//! "references.appears_in" → array of scene URIs
//! ```
//!
//! This is a first-class resolver capability — not a bolted-on feature.
//! It enables:
//! - AI systems to retrieve 500 tokens instead of 40,000
//! - Applications to fetch 10 KB instead of 5 MB
//! - CLI queries: `nap resolve nap://starwars/character/luke#references.appears_in`

use serde_yaml::Value;

use crate::error::NapError;

/// Query engine for extracting subtrees from manifest YAML values.
pub struct ManifestQuery;

impl ManifestQuery {
    /// Traverse a dot-separated path into a YAML value tree.
    ///
    /// # Arguments
    /// - `root` — The YAML value to traverse (typically a serialized Manifest).
    /// - `path` — Dot-separated path. e.g., `"references.appears_in"`.
    ///
    /// # Returns
    /// The subtree at the given path, or an error if any segment is missing.
    ///
    /// # Examples
    /// ```
    /// use serde_yaml::Value;
    /// use nap_core::query::ManifestQuery;
    ///
    /// let yaml = "
    /// name: Luke Skywalker
    /// properties:
    ///   homeworld: tatooine
    ///   species: human
    /// ";
    /// let root: Value = serde_yaml::from_str(yaml).unwrap();
    /// let result = ManifestQuery::query(&root, "properties.homeworld", "test").unwrap();
    /// assert_eq!(result, Value::String("tatooine".to_string()));
    /// ```
    pub fn query(root: &Value, path: &str, manifest_id: &str) -> Result<Value, NapError> {
        if path.is_empty() {
            return Err(NapError::InvalidQueryPath(
                "query path cannot be empty".to_string(),
            ));
        }

        let segments: Vec<&str> = path.split('.').collect();
        let mut current = root;

        for (depth, segment) in segments.iter().enumerate() {
            if segment.is_empty() {
                return Err(NapError::InvalidQueryPath(format!(
                    "empty segment at position {depth} in path '{path}'"
                )));
            }

            current = match current {
                Value::Mapping(map) => {
                    let key = Value::String(segment.to_string());
                    map.get(&key).ok_or_else(|| {
                        let traversed = segments[..depth].join(".");
                        let available_keys: Vec<String> = map
                            .keys()
                            .filter_map(|k| k.as_str().map(String::from))
                            .collect();
                        tracing::debug!(
                            manifest_id = manifest_id,
                            path = path,
                            segment = segment,
                            traversed = traversed,
                            available_keys = ?available_keys,
                            "query segment not found in mapping"
                        );
                        NapError::QueryPathNotFound {
                            path: path.to_string(),
                            manifest_id: manifest_id.to_string(),
                        }
                    })?
                }
                Value::Sequence(seq) => {
                    // Allow integer index access into sequences
                    let index: usize =
                        segment.parse().map_err(|_| NapError::QueryPathNotFound {
                            path: path.to_string(),
                            manifest_id: manifest_id.to_string(),
                        })?;
                    seq.get(index).ok_or_else(|| NapError::QueryPathNotFound {
                        path: path.to_string(),
                        manifest_id: manifest_id.to_string(),
                    })?
                }
                _ => {
                    return Err(NapError::QueryPathNotFound {
                        path: path.to_string(),
                        manifest_id: manifest_id.to_string(),
                    });
                }
            };
        }

        Ok(current.clone())
    }

    /// List available keys at a given path (for introspection / tab-completion).
    pub fn list_keys(root: &Value, path: &str, manifest_id: &str) -> Result<Vec<String>, NapError> {
        let target = if path.is_empty() {
            root.clone()
        } else {
            Self::query(root, path, manifest_id)?
        };

        match target {
            Value::Mapping(map) => Ok(map
                .keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect()),
            Value::Sequence(seq) => Ok((0..seq.len()).map(|i| i.to_string()).collect()),
            _ => Ok(vec![]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manifest() -> Value {
        let yaml = r#"
id: "nap://starwars/character/lukeskywalker"
name: "Luke Skywalker"
entity_type: character
version: 17
properties:
  homeworld: "nap://starwars/location/tatooine"
  species: human
  affiliation: rebel_alliance
representations:
  reference_image:
    hash: "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    format: png
references:
  appears_in:
    - "nap://starwars/scene/cantina"
    - "nap://starwars/scene/trench-run"
  relationships:
    - target: "nap://starwars/character/darthvader"
      type: parent
"#;
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn test_query_simple_property() {
        let root = test_manifest();
        let result = ManifestQuery::query(&root, "properties.species", "test").unwrap();
        assert_eq!(result, Value::String("human".to_string()));
    }

    #[test]
    fn test_query_nested_representation() {
        let root = test_manifest();
        let result =
            ManifestQuery::query(&root, "representations.reference_image.hash", "test").unwrap();
        assert_eq!(
            result,
            Value::String(
                "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_query_array() {
        let root = test_manifest();
        let result = ManifestQuery::query(&root, "references.appears_in", "test").unwrap();
        match result {
            Value::Sequence(seq) => assert_eq!(seq.len(), 2),
            _ => panic!("expected sequence"),
        }
    }

    #[test]
    fn test_query_array_index() {
        let root = test_manifest();
        let result = ManifestQuery::query(&root, "references.appears_in.0", "test").unwrap();
        assert_eq!(
            result,
            Value::String("nap://starwars/scene/cantina".to_string())
        );
    }

    #[test]
    fn test_query_not_found() {
        let root = test_manifest();
        let result = ManifestQuery::query(&root, "properties.nonexistent", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_query_empty_path() {
        let root = test_manifest();
        let result = ManifestQuery::query(&root, "", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_keys_root() {
        let root = test_manifest();
        let keys = ManifestQuery::list_keys(&root, "", "test").unwrap();
        assert!(keys.contains(&"properties".to_string()));
        assert!(keys.contains(&"representations".to_string()));
    }

    #[test]
    fn test_list_keys_nested() {
        let root = test_manifest();
        let keys = ManifestQuery::list_keys(&root, "properties", "test").unwrap();
        assert!(keys.contains(&"species".to_string()));
        assert!(keys.contains(&"homeworld".to_string()));
    }
}
