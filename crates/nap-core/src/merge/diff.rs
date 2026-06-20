//! Public diff API — user-facing change inspection.
//!
//! The merge engine itself does NOT depend on diff.
//! Diff is a presentation-layer utility for:
//!
//! - **Agent conflict resolution** — "show me what changed"
//! - **Portals UI** — React Flow display of additions/removals
//! - **Review workflows** — preview before publishing

use serde::Serialize;
use serde_json::Value;

use crate::merge::normalization::normalize;
use crate::merge::path::build_path_map;
use crate::merge::sdl::SdlDocument;

/// The result of a diff operation.
#[derive(Debug, Clone, Serialize)]
pub struct DiffResult {
    pub changes: Vec<Change>,
}

impl DiffResult {
    /// Returns `true` if there are no changes.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Returns the number of changes.
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Returns changes grouped by operation type.
    pub fn by_operation(&self) -> (Vec<&Change>, Vec<&Change>, Vec<&Change>) {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut modified = Vec::new();
        for c in &self.changes {
            match c.op {
                ChangeOp::Added => added.push(c),
                ChangeOp::Removed => removed.push(c),
                ChangeOp::Modified => modified.push(c),
            }
        }
        (added, removed, modified)
    }
}

/// A single change detected by the diff engine.
#[derive(Debug, Clone, Serialize)]
pub struct Change {
    /// The canonical path to the changed value.
    /// e.g. `"root.properties.homeworld"`
    pub path: String,

    /// The type of change.
    pub op: ChangeOp,

    /// The previous value (None for additions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_value: Option<Value>,

    /// The new value (None for removals).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_value: Option<Value>,
}

/// The kind of change operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeOp {
    /// A new field was added.
    Added,
    /// A field was removed (set to null).
    Removed,
    /// A field was modified.
    Modified,
}

/// Compute the diff between a base document and a candidate document.
///
/// # Normalization
///
/// The candidate is automatically normalized against the base before diffing.
/// This ensures that "missing = no change" is correctly interpreted.
///
/// # Paths
///
/// Paths use canonical form with identity keys when the SDL provides them.
///
/// # Arguments
///
/// * `base` - The reference document.
/// * `candidate` - The document to compare against base.
/// * `schema` - The SDL document (provides identity rules for path resolution).
///
/// # Returns
///
/// A `DiffResult` containing all detected changes.
pub fn diff(base: &Value, candidate: &Value, schema: &SdlDocument) -> DiffResult {
    // Build path maps (with SDL-aware identity resolution)
    let base_paths = build_path_map(base, schema);
    let candidate_paths = build_path_map(candidate, schema);

    let mut changes = Vec::new();

    // Find modified and removed paths
    for (path, base_val) in &base_paths {
        match candidate_paths.get(path) {
            None | Some(Value::Null) => {
                // Path missing or explicitly null in candidate
                let sub_path = format!("root.{path}");
                changes.push(Change {
                    path: sub_path,
                    op: ChangeOp::Removed,
                    old_value: Some(base_val.clone()),
                    new_value: None,
                });
            }
            Some(candidate_val) if candidate_val != base_val => {
                let sub_path = format!("root.{path}");
                changes.push(Change {
                    path: sub_path,
                    op: ChangeOp::Modified,
                    old_value: Some(base_val.clone()),
                    new_value: Some(candidate_val.clone()),
                });
            }
            _ => {
                // Values are equal — no change
            }
        }
    }

    // Find added paths (in candidate but not in base)
    for (path, candidate_val) in &candidate_paths {
        if !base_paths.contains_key(path) && !candidate_val.is_null() {
            let sub_path = format!("root.{path}");
            changes.push(Change {
                path: sub_path,
                op: ChangeOp::Added,
                old_value: None,
                new_value: Some(candidate_val.clone()),
            });
        }
    }

    // Sort by path for deterministic output
    changes.sort_by(|a, b| a.path.cmp(&b.path));

    DiffResult { changes }
}

/// Convenience: compute diff with automatic normalization.
///
/// Equivalent to:
/// ```ignore
/// let norm = normalize(base, candidate);
/// diff(base, &norm, schema)
/// ```
pub fn diff_normalized(base: &Value, candidate: &Value, schema: &SdlDocument) -> DiffResult {
    let normalized = normalize(base, candidate);
    diff(base, &normalized, schema)
}

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
"#,
        )
        .unwrap()
    }

    #[test]
    fn test_diff_modified() {
        let base = json!({"name": "Luke"});
        let candidate = json!({"name": "Luke Skywalker"});

        let result = diff(&base, &candidate, &test_sdl());
        assert_eq!(result.len(), 1);
        assert_eq!(result.changes[0].op, ChangeOp::Modified);
        assert_eq!(result.changes[0].path, "root.name");
        assert_eq!(result.changes[0].old_value, Some(json!("Luke")));
        assert_eq!(result.changes[0].new_value, Some(json!("Luke Skywalker")));
    }

    #[test]
    fn test_diff_added() {
        let base = json!({"name": "Luke"});
        let candidate = json!({"name": "Luke", "homeworld": "Tatooine"});

        let result = diff(&base, &candidate, &test_sdl());
        assert_eq!(result.len(), 1);
        assert_eq!(result.changes[0].op, ChangeOp::Added);
        assert_eq!(result.changes[0].path, "root.homeworld");
        assert_eq!(result.changes[0].old_value, None);
        assert_eq!(result.changes[0].new_value, Some(json!("Tatooine")));
    }

    #[test]
    fn test_diff_removed() {
        let base = json!({"name": "Luke", "homeworld": "Tatooine"});
        let candidate = json!({"name": "Luke", "homeworld": null});

        let result = diff(&base, &candidate, &test_sdl());
        assert_eq!(result.len(), 1);
        assert_eq!(result.changes[0].op, ChangeOp::Removed);
        assert_eq!(result.changes[0].path, "root.homeworld");
    }

    #[test]
    fn test_diff_no_changes() {
        let base = json!({"name": "Luke"});
        let candidate = json!({"name": "Luke"});

        let result = diff(&base, &candidate, &test_sdl());
        assert!(result.is_empty());
    }

    #[test]
    fn test_diff_by_operation() {
        let base = json!({"a": 1, "b": 2});
        let candidate = json!({"a": 10, "c": 3});

        let result = diff(&base, &candidate, &test_sdl());
        let (added, removed, modified) = result.by_operation();
        assert_eq!(added.len(), 1); // c added
        assert_eq!(removed.len(), 1); // b removed
        assert_eq!(modified.len(), 1); // a modified
    }

    #[test]
    fn test_diff_deterministic() {
        let base = json!({"b": 2, "a": 1});
        let candidate = json!({"a": 10, "c": 3});

        let result1 = diff(&base, &candidate, &test_sdl());
        let result2 = diff(&base, &candidate, &test_sdl());
        assert_eq!(result1.changes.len(), result2.changes.len());
        // Paths should be sorted consistently
        for (c1, c2) in result1.changes.iter().zip(result2.changes.iter()) {
            assert_eq!(c1.path, c2.path);
        }
    }
}
