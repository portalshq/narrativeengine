//! Three-way merge engine — path-map reconciliation.
//!
//! The engine operates on `serde_json::Value` (JSON AST), never on YAML
//! text.  It implements the protocol invariants defined in
//! `merge-semantics-v2.md`:
//!
//! 1. Normalize before merge
//! 2. Missing ≠ null
//! 3. Identity immutable
//! 4. Merge over path union
//! 5. Validate before persist (caller's responsibility)
//! 6. Validate after merge (caller's responsibility)
//! 7. Deterministic execution
//!
//! The engine does NOT depend on the `diff` module.

use serde_json::Value;

use crate::merge::conflict::{Conflict, MergeResult};
use crate::merge::normalization::normalize;
use crate::merge::path::{CanonicalPath, build_path_map, path_union};
use crate::merge::sdl::{IdentityRule, MergeStrategyType, PropertyDef, SdlDocument};
use crate::merge::strategies;

/// The three-way merge engine.
///
/// Construct with an SDL document, then call `merge()` with
/// base/current/proposed values.
#[derive(Debug, Clone)]
pub struct MergeEngine {
    schema: SdlDocument,
}

impl MergeEngine {
    /// Create a new merge engine from an SDL document.
    ///
    /// The SDL document must be valid (call `validate::sdl::validate_sdl()`
    /// first if needed).
    pub fn new(schema: SdlDocument) -> Self {
        MergeEngine { schema }
    }

    /// Return a reference to the SDL schema.
    pub fn schema(&self) -> &SdlDocument {
        &self.schema
    }

    /// Perform a three-way merge.
    ///
    /// # Pipeline
    ///
    /// 1. Normalize current and proposed against base.
    /// 2. Build path maps for all three documents.
    /// 3. Compute the union of all paths.
    /// 4. For each path, resolve values and apply merge strategy.
    /// 5. Check for identity mutations (protocol invariant).
    /// 6. Return merged result or conflicts.
    ///
    /// # Arguments
    ///
    /// * `base` - The base (reference) document.
    /// * `current` - The current (our) document.
    /// * `proposed` - The proposed (their) document.
    ///
    /// # Returns
    ///
    /// `MergeResult::Merged(Value)` on success, or
    /// `MergeResult::Conflicts(Vec<Conflict>)` if conflicts were found.
    pub fn merge(&self, base: Value, current: Value, proposed: Value) -> MergeResult {
        // Step 1: Normalize
        let normalized_current = normalize(&base, &current);
        let normalized_proposed = normalize(&base, &proposed);

        // Step 2: Build path maps
        let base_paths = build_path_map(&base, &self.schema);
        let current_paths = build_path_map(&normalized_current, &self.schema);
        let proposed_paths = build_path_map(&normalized_proposed, &self.schema);

        // Step 3: Path union
        let all_paths = path_union(&[&base_paths, &current_paths, &proposed_paths]);

        // Step 4: For each path, apply merge strategy
        let mut merged = base.clone();
        let mut all_conflicts = Vec::new();

        for path in &all_paths {
            // Skip sub-paths of identity-keyed arrays — the array-level
            // strategy handles those items as a whole.
            if self.is_identity_array_subpath(path) {
                continue;
            }

            let base_val = base_paths.get(path).cloned().unwrap_or(Value::Null);
            let current_val = current_paths.get(path).cloned().unwrap_or(Value::Null);
            let proposed_val = proposed_paths.get(path).cloned().unwrap_or(Value::Null);

            // Skip paths where all three are identical
            if base_val == current_val && current_val == proposed_val {
                continue;
            }

            // Determine the SDL property path (strip root. prefix if present)
            let sdl_path = path.strip_prefix("root.").unwrap_or(path);

            // Look up merge strategy from SDL
            let result = match self.schema.property_def(sdl_path) {
                Some(def) => {
                    // Check for identity mutation first (protocol invariant)
                    if let Some(mutation_conflict) = self.check_identity_mutation(
                        sdl_path,
                        def,
                        &base_val,
                        &current_val,
                        &proposed_val,
                    ) {
                        MergeResult::Conflicts(vec![mutation_conflict])
                    } else {
                        // Apply the declared strategy
                        self.apply_strategy(sdl_path, def, &base_val, &current_val, &proposed_val)
                    }
                }
                None => {
                    // No SDL definition for this path → treat as replace
                    // (Validation layer should warn/error on schema-less properties,
                    // but the engine must still produce a deterministic result.)
                    strategies::replace::merge_replace(
                        &format!("root.{sdl_path}"),
                        &base_val,
                        &current_val,
                        &proposed_val,
                    )
                }
            };

            // Update the merged document or collect conflicts
            match result {
                MergeResult::Merged(val) => {
                    // Insert the merged value into the result
                    if let Err(e) = set_value_at_path(&mut merged, path, val) {
                        // If we can't set the value at this path, it's a conflict
                        all_conflicts.push(Conflict::structural(
                            format!("root.{sdl_path}"),
                            base_val,
                            current_val,
                            proposed_val,
                        ));
                        // Log the error for debugging
                        tracing::debug!(
                            "merge_engine: failed to set value at path '{}': {}",
                            path,
                            e
                        );
                    }
                }
                MergeResult::Conflicts(mut cs) => {
                    all_conflicts.append(&mut cs);
                }
            }
        }

        // Step 5: Handle paths not covered by path map iteration
        // This includes top-level deletions (when the entire document is stripped)
        // which are handled implicitly by the iteration above.

        if all_conflicts.is_empty() {
            MergeResult::Merged(merged)
        } else {
            MergeResult::Conflicts(all_conflicts)
        }
    }

    /// Check if a path is a sub-path of an identity-keyed array.
    ///
    /// For example, if `characters` is an `ordered_unique` array with
    /// `identity: {key: id}`, then `characters[obiwan]` and
    /// `characters[obiwan].name` are sub-paths that should be skipped
    /// during merge — the array-level strategy handles them.
    fn is_identity_array_subpath(&self, path: &str) -> bool {
        let parsed = match CanonicalPath::parse(path) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let segments = parsed.segments();
        if segments.len() <= 1 {
            return false;
        }

        // Check if any parent segment is an Identity segment
        // (meaning we're inside a specific array item)
        for (i, seg) in segments.iter().enumerate() {
            if matches!(seg, crate::merge::path::PathSegment::Identity(_)) {
                // Found an identity segment — check if it's not the last segment,
                // or if the parent array path is an SDL-defined identity array.
                let parent_path = segments[..i]
                    .iter()
                    .map(|s| match s {
                        crate::merge::path::PathSegment::Key(k) => k.clone(),
                        crate::merge::path::PathSegment::Identity(id) => format!("[{id}]"),
                    })
                    .collect::<Vec<_>>()
                    .join(".");

                // If the parent array path is defined in SDL with an identity strategy,
                // this is a sub-path to be skipped.
                if let Some(def) = self.schema.property_def(&parent_path)
                    && matches!(
                        def.merge.strategy_type,
                        MergeStrategyType::OrderedUnique
                            | MergeStrategyType::SetUnion
                            | MergeStrategyType::EdgeList
                    )
                {
                    return true;
                }

                // Also check if this is an array item path itself (identity is last segment)
                // by checking if the path without the parent is just an identity.
                if i == segments.len() - 1 {
                    return true; // It's tags[a] or similar — skip individual item
                }
            }
        }

        false
    }

    /// Check for identity mutation conflicts.
    ///
    /// Returns `Some(Conflict)` if the identity of an array element was mutated,
    /// which is forbidden by the protocol.
    ///
    /// Detection strategy: compare items at the same **position** in the array
    /// and check whether their identity key values differ.  If the item at
    /// position 0 in base has `id: "obiwan"` and the item at position 0 in
    /// current has `id: "ben_kenobi"`, that is a mutation even though a
    /// by-identity-indexed map would see two unrelated items.
    fn check_identity_mutation(
        &self,
        path: &str,
        def: &PropertyDef,
        base: &Value,
        current: &Value,
        proposed: &Value,
    ) -> Option<Conflict> {
        // Only applicable for array strategies with identity rules
        let identity = match &def.merge.identity {
            Some(id) => id,
            None => return None,
        };

        let identity_key = match identity {
            IdentityRule::Key { key } => key,
            IdentityRule::PrimitiveValue => return None, // primitive value IS the identity, can't change
        };

        // Check that all three values are arrays
        let base_arr = base.as_array()?;
        let current_arr = current.as_array()?;
        let proposed_arr = proposed.as_array()?;

        // Check current against base: compare identity keys at each position
        for i in 0..base_arr.len().min(current_arr.len()) {
            let base_id = base_arr[i].get(identity_key);
            let cur_id = current_arr[i].get(identity_key);
            if base_id != cur_id {
                let base_id_str = base_id.and_then(|v| v.as_str()).unwrap_or("?");
                let sub_path = format!("root.{path}[{base_id_str}]");
                let proposed_val = proposed_arr.get(i).cloned().unwrap_or(Value::Null);
                return Some(Conflict::identity_mutation(
                    sub_path,
                    base_arr[i].clone(),
                    current_arr[i].clone(),
                    proposed_val,
                ));
            }
        }

        // Check proposed against base: compare identity keys at each position
        for i in 0..base_arr.len().min(proposed_arr.len()) {
            let base_id = base_arr[i].get(identity_key);
            let prop_id = proposed_arr[i].get(identity_key);
            if base_id != prop_id {
                let base_id_str = base_id.and_then(|v| v.as_str()).unwrap_or("?");
                let sub_path = format!("root.{path}[{base_id_str}]");
                let current_val = current_arr.get(i).cloned().unwrap_or(Value::Null);
                return Some(Conflict::identity_mutation(
                    sub_path,
                    base_arr[i].clone(),
                    current_val,
                    proposed_arr[i].clone(),
                ));
            }
        }

        None
    }

    /// Apply the appropriate merge strategy for a path.
    fn apply_strategy(
        &self,
        path: &str,
        def: &PropertyDef,
        base: &Value,
        current: &Value,
        proposed: &Value,
    ) -> MergeResult {
        let full_path = format!("root.{path}");

        match def.merge.strategy_type {
            MergeStrategyType::Replace => {
                strategies::replace::merge_replace(&full_path, base, current, proposed)
            }

            MergeStrategyType::DeepMerge => {
                strategies::deep_merge::merge_deep(&full_path, base, current, proposed)
            }

            MergeStrategyType::Atomic => {
                strategies::atomic::merge_atomic(&full_path, base, current, proposed)
            }

            MergeStrategyType::OrderedUnique => {
                let identity = def
                    .merge
                    .identity
                    .clone()
                    .unwrap_or(IdentityRule::PrimitiveValue);
                strategies::ordered_unique::merge_ordered_unique(
                    &full_path, base, current, proposed, &identity,
                )
            }

            MergeStrategyType::SetUnion => {
                let identity = def
                    .merge
                    .identity
                    .clone()
                    .unwrap_or(IdentityRule::PrimitiveValue);
                strategies::set_union::merge_set_union(
                    &full_path, base, current, proposed, &identity,
                )
            }

            MergeStrategyType::EdgeList => {
                let identity = def.merge.identity.clone().unwrap_or(IdentityRule::Key {
                    key: "id".to_string(),
                });
                let source_key = def
                    .merge
                    .source_key
                    .clone()
                    .unwrap_or_else(|| "source".to_string());
                let target_key = def
                    .merge
                    .target_key
                    .clone()
                    .unwrap_or_else(|| "target".to_string());
                strategies::edge_list::merge_edge_list(
                    &full_path,
                    base,
                    current,
                    proposed,
                    &identity,
                    &source_key,
                    &target_key,
                )
            }
        }
    }
}

/// Set a value at a given canonical path within a JSON document.
///
/// Creates intermediate objects as needed.
/// Returns an error if the path cannot be set (e.g., type conflict).
fn set_value_at_path(root: &mut Value, path: &str, value: Value) -> Result<(), String> {
    let canonical =
        CanonicalPath::parse(path).map_err(|e| format!("invalid path '{path}': {e}"))?;

    let segments = canonical.segments().to_vec();
    if segments.is_empty() {
        return Err("empty path".to_string());
    }

    // Navigate to the parent of the final segment
    let parent_segments = &segments[..segments.len() - 1];
    let last_segment = &segments[segments.len() - 1];

    let mut current = root;

    // Navigate/build intermediate segments
    for segment in parent_segments {
        match segment {
            crate::merge::path::PathSegment::Key(key) => {
                if !current.is_object() {
                    return Err(format!("cannot enter non-object at '{key}'"));
                }
                current = current
                    .as_object_mut()
                    .unwrap()
                    .entry(key.clone())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
            }
            crate::merge::path::PathSegment::Identity(id) => {
                // For identity segments, we assume the array already has the item
                // (it was created during path map building)
                if let Some(item) = find_item_by_identity(current, id) {
                    current = item;
                } else {
                    return Err(format!("identity '{id}' not found in array"));
                }
            }
        }
    }

    // Set the value at the final segment
    match last_segment {
        crate::merge::path::PathSegment::Key(key) => {
            if let Value::Object(map) = current {
                map.insert(key.clone(), value);
                Ok(())
            } else {
                Err(format!("cannot set key '{key}' on non-object"))
            }
        }
        crate::merge::path::PathSegment::Identity(id) => {
            // Find the item by identity and replace it
            if let Some(item) = find_item_by_identity(current, id) {
                *item = value;
                Ok(())
            } else {
                Err(format!("identity '{id}' not found in array"))
            }
        }
    }
}

/// Find an item in an array by matching its identity value (any string field).
fn find_item_by_identity<'a>(root: &'a mut Value, identity: &str) -> Option<&'a mut Value> {
    match root {
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                if has_identity(item, identity) {
                    return Some(item);
                }
            }
            None
        }
        _ => None,
    }
}

/// Check whether a JSON value matches a given identity string.
fn has_identity(item: &Value, identity: &str) -> bool {
    match item {
        Value::Object(map) => map.values().any(|v| v.as_str() == Some(identity)),
        Value::String(s) => s == identity,
        Value::Number(n) => n.to_string() == identity,
        Value::Bool(b) => b.to_string() == identity,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn simple_sdl() -> SdlDocument {
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
    version:
      type: number
      merge:
        type: atomic
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
    edges:
      type: array
      merge:
        type: edge_list
        source_key: source
        target_key: target
        identity:
          mode: key
          key: id
"#,
        )
        .unwrap()
    }

    #[test]
    fn test_merge_simple_replace() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({"name": "Luke"});
        let current = json!({"name": "Luke Skywalker"});
        let proposed = json!({"name": "Luke"});

        let result = engine.merge(base, current, proposed);
        assert!(result.is_merged());
        assert_eq!(
            result.unwrap_merged().get("name"),
            Some(&json!("Luke Skywalker"))
        );
    }

    #[test]
    fn test_merge_replace_conflict() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({"name": "Luke"});
        let current = json!({"name": "Luke Skywalker"});
        let proposed = json!({"name": "Anakin"});

        let result = engine.merge(base, current, proposed);
        assert!(result.is_conflict());
    }

    #[test]
    fn test_merge_missing_field_preserved() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({"name": "Obi Wan", "version": 1});
        let current = json!({"name": "Obi Wan Kenobi"});
        let proposed = json!({"name": "Obi Wan"});

        // Normalization should fill in version from base
        let result = engine.merge(base, current, proposed);
        assert!(result.is_merged());
        let merged = result.unwrap_merged();
        assert_eq!(merged.get("name"), Some(&json!("Obi Wan Kenobi")));
        assert_eq!(merged.get("version"), Some(&json!(1)));
    }

    #[test]
    fn test_merge_null_is_deletion() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({"name": "Luke", "version": 1});
        let current = json!({"name": "Luke", "version": 1});
        let proposed = json!({"name": "Luke", "version": null});

        let result = engine.merge(base, current, proposed);
        assert!(result.is_merged());
        let merged = result.unwrap_merged();
        // version should be null (explicit deletion)
        assert_eq!(merged.get("version"), Some(&Value::Null));
    }

    #[test]
    fn test_merge_ordered_unique_objects() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({"characters": [{"id": "A", "name": "Alpha"}]});
        let current =
            json!({"characters": [{"id": "A", "name": "Alpha"}, {"id": "B", "name": "Beta"}]});
        let proposed =
            json!({"characters": [{"id": "A", "name": "Alpha"}, {"id": "C", "name": "Gamma"}]});

        let result = engine.merge(base, current, proposed);
        assert!(result.is_merged());
        let merged = result.unwrap_merged();
        let chars = merged["characters"].as_array().unwrap();
        assert_eq!(chars.len(), 3);
        assert_eq!(chars[0]["id"], json!("A"));
        assert_eq!(chars[1]["id"], json!("B"));
        assert_eq!(chars[2]["id"], json!("C"));
    }

    #[test]
    fn test_merge_ordered_unique_primitives() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({"tags": ["a", "b"]});
        let current = json!({"tags": ["a", "b", "c"]});
        let proposed = json!({"tags": ["a", "b", "d"]});

        let result = engine.merge(base, current, proposed);
        assert!(result.is_merged());
        let merged = result.unwrap_merged();
        assert_eq!(
            merged["tags"].as_array().unwrap(),
            &[json!("a"), json!("b"), json!("c"), json!("d")]
        );
    }

    #[test]
    fn test_merge_atomic_conflict() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({"version": 1});
        let current = json!({"version": 2});
        let proposed = json!({"version": 3});

        let result = engine.merge(base, current, proposed);
        assert!(result.is_conflict());
    }

    #[test]
    fn test_merge_edge_list() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({"edges": [{"id": "e1", "source": "a", "target": "b"}]});
        let current = json!({
            "edges": [
                {"id": "e1", "source": "a", "target": "b"},
                {"id": "e2", "source": "b", "target": "c"}
            ]
        });
        let proposed = json!({
            "edges": [
                {"id": "e1", "source": "a", "target": "b"},
                {"id": "e3", "source": "c", "target": "a"}
            ]
        });

        let result = engine.merge(base, current, proposed);
        assert!(result.is_merged());
        let merged = result.unwrap_merged();
        assert_eq!(merged["edges"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_merge_identity_mutation_conflict() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({"characters": [{"id": "obiwan", "name": "Obi-Wan"}]});
        let current = json!({"characters": [{"id": "ben_kenobi", "name": "Obi-Wan"}]}); // id changed!
        let proposed = json!({"characters": [{"id": "obiwan", "name": "Obi-Wan"}]});

        let result = engine.merge(base, current, proposed);
        assert!(result.is_conflict());
    }

    #[test]
    fn test_merge_deterministic() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({"name": "Luke", "version": 1, "tags": ["a"]});
        let current = json!({"name": "Luke Skywalker", "version": 2, "tags": ["a", "b"]});
        let proposed = json!({"name": "Luke", "version": 1, "tags": ["a", "c"]});

        let result1 = engine.merge(base.clone(), current.clone(), proposed.clone());
        let result2 = engine.merge(base, current, proposed);

        // Both should produce identical results
        match (result1, result2) {
            (MergeResult::Merged(a), MergeResult::Merged(b)) => assert_eq!(a, b),
            (MergeResult::Conflicts(a), MergeResult::Conflicts(b)) => {
                assert_eq!(a.len(), b.len());
                for (ca, cb) in a.iter().zip(b.iter()) {
                    assert_eq!(ca.path, cb.path);
                }
            }
            _ => panic!("results should be same variant"),
        }
    }

    #[test]
    fn test_merge_empty_documents() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({});
        let current = json!({});
        let proposed = json!({});

        let result = engine.merge(base, current, proposed);
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!({}));
    }

    #[test]
    fn test_merge_additions_from_both_sides() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({});
        let current = json!({"name": "From Current"});
        let proposed = json!({"version": 42});

        let result = engine.merge(base, current, proposed);
        assert!(result.is_merged());
        let merged = result.unwrap_merged();
        assert_eq!(merged.get("name"), Some(&json!("From Current")));
        assert_eq!(merged.get("version"), Some(&json!(42)));
    }

    #[test]
    fn test_merge_both_add_same_field_conflict() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({});
        let current = json!({"name": "From Current"});
        let proposed = json!({"name": "From Proposed"});

        let result = engine.merge(base, current, proposed);
        // Both changed "name" differently from base → conflict
        assert!(result.is_conflict());
    }

    #[test]
    fn test_merge_ordered_unique_modification_accepted() {
        let engine = MergeEngine::new(simple_sdl());

        let base = json!({"characters": [{"id": "A", "val": 1}]});
        let current = json!({"characters": [{"id": "A", "val": 2}]}); // modified
        let proposed = json!({"characters": [{"id": "A", "val": 1}]}); // same as base

        let result = engine.merge(base, current, proposed);
        assert!(result.is_merged());
        let merged = result.unwrap_merged();
        assert_eq!(merged["characters"][0]["val"], json!(2));
    }

    // ── Property-style determinism test ────────────────────────────────

    /// Generate random documents and verify that merge is deterministic.
    #[test]
    fn test_deterministic_random_iterations() {
        use rand::Rng;
        let engine = MergeEngine::new(simple_sdl());
        let mut rng = rand::rng();

        for _ in 0..50 {
            // Build a simple random document
            let mut base = serde_json::Map::new();
            let mut current = serde_json::Map::new();
            let mut proposed = serde_json::Map::new();

            // Add random fields
            let field_count = rng.random_range(0..6);
            for i in 0..field_count {
                let key = format!("field_{i}");
                let base_val = rng.random_range(0..100);
                let cur_val = if rng.random_bool(0.5) {
                    base_val + rng.random_range(-5..=5)
                } else {
                    base_val
                };
                let prop_val = if rng.random_bool(0.5) {
                    base_val + rng.random_range(-5..=5)
                } else {
                    base_val
                };
                base.insert(key.clone(), json!(base_val));
                current.insert(key.clone(), json!(cur_val));
                proposed.insert(key.clone(), json!(prop_val));
            }

            // Add random tags
            let tag_count = rng.random_range(0..4);
            let mut tags: Vec<Value> = (0..tag_count).map(|i| json!(format!("tag_{i}"))).collect();
            if rng.random_bool(0.3) {
                tags.push(json!("tag_0")); // intentional duplicate
            }
            if !tags.is_empty() {
                base.insert("tags".to_string(), Value::Array(tags.clone()));
                if rng.random_bool(0.5) {
                    tags.push(json!("tag_new"));
                }
                current.insert("tags".to_string(), Value::Array(tags.clone()));
                if rng.random_bool(0.5) {
                    tags.push(json!("tag_extra"));
                }
                proposed.insert("tags".to_string(), Value::Array(tags));
            }

            let base_val = Value::Object(base);
            let current_val = Value::Object(current);
            let proposed_val = Value::Object(proposed);

            // Run merge twice
            let result1 = engine.merge(base_val.clone(), current_val.clone(), proposed_val.clone());
            let result2 = engine.merge(base_val, current_val, proposed_val);

            // Verify determinism
            match (&result1, &result2) {
                (MergeResult::Merged(a), MergeResult::Merged(b)) => {
                    assert_eq!(a, b, "deterministic merge failed");
                }
                (MergeResult::Conflicts(a), MergeResult::Conflicts(b)) => {
                    assert_eq!(a.len(), b.len(), "different conflict counts");
                    for (ca, cb) in a.iter().zip(b.iter()) {
                        assert_eq!(ca.path, cb.path, "different conflict paths");
                    }
                }
                _ => panic!("merge results differ in type"),
            }
        }
    }
}
