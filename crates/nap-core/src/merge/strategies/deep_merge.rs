//! Deep merge strategy.
//!
//! Recursive object merge.  For each sub-path, the standard conflict
//! matrix is applied.  Only valid for `object`-typed properties.

use serde_json::Value;

use crate::merge::conflict::{Conflict, MergeResult};

/// Merge two objects recursively.
///
/// For each key in the union of current and proposed:
/// 1. Resolve base, current, proposed values for that sub-key.
/// 2. Apply the conflict matrix:
///    - `current == base && proposed != base` → accept proposed
///    - `proposed == base && current != base` → accept current
///    - `current == proposed` → accept either
///    - all equal → accept base
///    - divergent → conflict
///
/// Returns `MergeResult::Conflicts` if any sub-key produces a conflict.
pub fn merge_deep(path: &str, base: &Value, current: &Value, proposed: &Value) -> MergeResult {
    // If any value is not an object, fall back to replace semantics.
    if !base.is_object() || !current.is_object() || !proposed.is_object() {
        return deep_fallback(path, base, current, proposed);
    }

    let base_obj = base.as_object().unwrap();
    let current_obj = current.as_object().unwrap();
    let proposed_obj = proposed.as_object().unwrap();

    // Collect union of all keys
    let mut all_keys: Vec<String> = Vec::new();
    for key in base_obj
        .keys()
        .chain(current_obj.keys())
        .chain(proposed_obj.keys())
    {
        if !all_keys.contains(key) {
            all_keys.push(key.clone());
        }
    }
    all_keys.sort();

    let mut merged = serde_json::Map::new();
    let mut all_conflicts = Vec::new();

    for key in &all_keys {
        let sub_path = if path.is_empty() {
            key.clone()
        } else {
            format!("{path}.{key}")
        };

        let base_val = base_obj.get(key.as_str()).cloned().unwrap_or(Value::Null);
        let current_val = current_obj
            .get(key.as_str())
            .cloned()
            .unwrap_or(Value::Null);
        let proposed_val = proposed_obj
            .get(key.as_str())
            .cloned()
            .unwrap_or(Value::Null);

        // Recurse into deep_merge if all three are objects
        let sub_result =
            if base_val.is_object() && current_val.is_object() && proposed_val.is_object() {
                merge_deep(&sub_path, &base_val, &current_val, &proposed_val)
            } else {
                // Standard conflict matrix
                apply_matrix(&sub_path, &base_val, &current_val, &proposed_val)
            };

        match sub_result {
            MergeResult::Merged(v) => {
                merged.insert(key.clone(), v);
            }
            MergeResult::Conflicts(mut cs) => {
                all_conflicts.append(&mut cs);
            }
        }
    }

    if all_conflicts.is_empty() {
        MergeResult::Merged(Value::Object(merged))
    } else {
        MergeResult::Conflicts(all_conflicts)
    }
}

/// Fallback for when not all values are objects.
fn deep_fallback(path: &str, base: &Value, current: &Value, proposed: &Value) -> MergeResult {
    if current == base && proposed == base {
        return MergeResult::Merged(base.clone());
    }
    if current != base && proposed == base {
        return MergeResult::Merged(current.clone());
    }
    if current == base && proposed != base {
        return MergeResult::Merged(proposed.clone());
    }
    if current == proposed {
        return MergeResult::Merged(current.clone());
    }
    MergeResult::Conflicts(vec![Conflict::structural(
        path,
        base.clone(),
        current.clone(),
        proposed.clone(),
    )])
}

/// Apply the conflict matrix for a leaf path.
fn apply_matrix(path: &str, base: &Value, current: &Value, proposed: &Value) -> MergeResult {
    if current == base && proposed == base {
        return MergeResult::Merged(base.clone());
    }
    if current != base && proposed == base {
        return MergeResult::Merged(current.clone());
    }
    if current == base && proposed != base {
        return MergeResult::Merged(proposed.clone());
    }
    if current == proposed {
        return MergeResult::Merged(current.clone());
    }
    MergeResult::Conflicts(vec![Conflict::value_mismatch(
        path,
        base.clone(),
        current.clone(),
        proposed.clone(),
    )])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deep_merge_accepts_changes_from_both_sides() {
        let base = json!({"a": 1, "b": 2, "c": 3});
        let current = json!({"a": 10, "b": 2, "c": 3});
        let proposed = json!({"a": 1, "b": 20, "c": 3});

        let result = merge_deep("root", &base, &current, &proposed);
        assert!(result.is_merged());
        let merged = result.unwrap_merged();
        assert_eq!(merged.get("a"), Some(&json!(10)));
        assert_eq!(merged.get("b"), Some(&json!(20)));
        assert_eq!(merged.get("c"), Some(&json!(3)));
    }

    #[test]
    fn test_deep_merge_nested_objects() {
        let base = json!({"outer": {"inner_a": 1, "inner_b": 2}});
        let current = json!({"outer": {"inner_a": 10, "inner_b": 2}});
        let proposed = json!({"outer": {"inner_a": 1, "inner_b": 20}});

        let result = merge_deep("root", &base, &current, &proposed);
        assert!(result.is_merged());
        let merged = result.unwrap_merged();
        assert_eq!(merged.pointer("/outer/inner_a"), Some(&json!(10)));
        assert_eq!(merged.pointer("/outer/inner_b"), Some(&json!(20)));
    }

    #[test]
    fn test_deep_merge_conflict() {
        let base = json!({"a": 1, "b": 2});
        let current = json!({"a": 10, "b": 2});
        let proposed = json!({"a": 20, "b": 2});

        let result = merge_deep("root", &base, &current, &proposed);
        assert!(result.is_conflict());
    }

    #[test]
    fn test_deep_merge_additions_preserved() {
        let base = json!({"a": 1});
        let current = json!({"a": 1, "b": 2});
        let proposed = json!({"a": 1, "c": 3});

        let result = merge_deep("root", &base, &current, &proposed);
        assert!(result.is_merged());
        let merged = result.unwrap_merged();
        assert_eq!(merged.get("a"), Some(&json!(1)));
        assert_eq!(merged.get("b"), Some(&json!(2)));
        assert_eq!(merged.get("c"), Some(&json!(3)));
    }

    #[test]
    fn test_deep_merge_additions_conflict() {
        let base = json!({"a": 1});
        let current = json!({"a": 1, "new_key": "from_current"});
        let proposed = json!({"a": 1, "new_key": "from_proposed"});

        let result = merge_deep("root", &base, &current, &proposed);
        assert!(result.is_conflict());
    }

    #[test]
    fn test_non_object_falls_back() {
        let base = json!("string");
        let current = json!("changed");
        let proposed = json!("string");

        let result = merge_deep("root", &base, &current, &proposed);
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!("changed"));
    }

    #[test]
    fn test_deep_merge_all_equal() {
        let base = json!({"x": 1, "y": 2});
        let current = json!({"x": 1, "y": 2});
        let proposed = json!({"x": 1, "y": 2});

        let result = merge_deep("root", &base, &current, &proposed);
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), base);
    }
}
