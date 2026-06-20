//! Ordered unique merge strategy.
//!
//! For ordered lists where element identity matters:
//! - Preserves base order.
//! - Appends new items from current and proposed that are not in base.
//! - Deduplicates by identity.
//! - Conflicts when both branches modify the same identity differently.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::merge::conflict::{Conflict, MergeResult};
use crate::merge::sdl::IdentityRule;

/// Ordered-unique merge for an array.
///
/// # Algorithm
///
/// 1. Build an identity-indexed map for base, current, proposed.
/// 2. Preserve base order — walk base items in order.
/// 3. Append items from current that are new.
/// 4. Append items from proposed that are new.
/// 5. If any identity exists in both current and proposed with different
///    values (and neither matches base), conflict.
///
/// # Arguments
///
/// * `path` - Canonical path for diagnostics.
/// * `base` - Base array.
/// * `current` - Current (our) array.
/// * `proposed` - Proposed (their) array.
/// * `identity` - Identity rule (primitive_value or key).
pub fn merge_ordered_unique(
    path: &str,
    base: &Value,
    current: &Value,
    proposed: &Value,
    identity: &IdentityRule,
) -> MergeResult {
    // Ensure all values are arrays
    let base_arr = as_array(base);
    let current_arr = as_array(current);
    let proposed_arr = as_array(proposed);

    // Build identity-indexed maps
    let base_by_id = index_by_identity(base_arr, identity);
    let current_by_id = index_by_identity(current_arr, identity);
    let proposed_by_id = index_by_identity(proposed_arr, identity);

    // Check for identity conflicts: same identity modified differently
    let mut conflicts = Vec::new();

    for (id, cur_val) in &current_by_id {
        if let Some(pro_val) = proposed_by_id.get(id)
            && cur_val != pro_val
        {
            let base_val = base_by_id.get(id);
            // Check if one matches base (non-conflicting)
            let cur_matches_base = base_val == Some(cur_val);
            let pro_matches_base = base_val == Some(pro_val);

            if !cur_matches_base && !pro_matches_base {
                // Neither matches base → actual conflict
                let sub_path = format!("{path}[{id}]");
                conflicts.push(Conflict::value_mismatch(
                    sub_path,
                    base_val.cloned().unwrap_or(Value::Null),
                    cur_val.clone(),
                    pro_val.clone(),
                ));
            }
        }
    }

    if !conflicts.is_empty() {
        return MergeResult::Conflicts(conflicts);
    }

    // Build merged array preserving base order, then appending new items
    let mut seen: BTreeMap<String, bool> = BTreeMap::new();
    let mut result: Vec<Value> = Vec::new();

    // Helper to add an item if not already seen
    let mut add_item = |val: &Value, id: &str| {
        if !seen.contains_key(id) {
            seen.insert(id.to_string(), true);
            result.push(val.clone());
        }
    };

    // 1. Add all base items (preserves base order)
    for val in base_arr {
        let id = extract_identity(val, identity);
        add_item(val, &id);
    }

    // 2. Append new items from current
    for val in current_arr {
        let id = extract_identity(val, identity);
        if !base_by_id.contains_key(&id) {
            add_item(val, &id);
        }
    }

    // 3. Append new items from proposed
    for val in proposed_arr {
        let id = extract_identity(val, identity);
        if !base_by_id.contains_key(&id) && !current_by_id.contains_key(&id) {
            add_item(val, &id);
        }
    }

    // Items modified by current or proposed (not in base) are handled via
    // `apply_modifications` below.  The conflict check above already caught
    // divergent modifications.

    // For items that exist in base but were modified by current or proposed,
    // use the modified version.
    apply_modifications(
        &mut result,
        base_arr,
        &current_by_id,
        &proposed_by_id,
        identity,
    );

    MergeResult::Merged(Value::Array(result))
}

/// Apply modifications from current/proposed to base items in the result.
fn apply_modifications(
    result: &mut [Value],
    base_arr: &[Value],
    current_by_id: &BTreeMap<String, Value>,
    proposed_by_id: &BTreeMap<String, Value>,
    identity: &IdentityRule,
) {
    for (idx, val) in base_arr.iter().enumerate() {
        let id = extract_identity(val, identity);
        // Check if current modified this item
        if let Some(cur_val) = current_by_id.get(&id)
            && cur_val != val
        {
            result[idx] = cur_val.clone();
        }
        // Check if proposed modified this item (only if current didn't,
        // since identity conflicts already caught)
        if let Some(pro_val) = proposed_by_id.get(&id)
            && pro_val != &result[idx]
            && !current_by_id.contains_key(&id)
        {
            result[idx] = pro_val.clone();
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────

/// Extract the identity string from a value according to the identity rule.
fn extract_identity(val: &Value, identity: &IdentityRule) -> String {
    match identity {
        IdentityRule::PrimitiveValue => match val {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => {
                // Complex value as identity — serialize to JSON
                serde_json::to_string(val).unwrap_or_else(|_| "unknown".to_string())
            }
        },
        IdentityRule::Key { key } => val
            .get(key)
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                _ => None,
            })
            .unwrap_or_else(|| {
                // Fallback: use "unknown" for missing identity key
                // Validation layer should catch this.
                "unknown".to_string()
            }),
    }
}

/// Index an array by identity, returning a map from identity string → value.
fn index_by_identity(arr: &[Value], identity: &IdentityRule) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    for val in arr {
        let id = extract_identity(val, identity);
        map.insert(id, val.clone());
    }
    map
}

/// Convert a JSON value to an array slice, or return an empty slice.
fn as_array(val: &Value) -> &[Value] {
    match val {
        Value::Array(arr) => arr.as_slice(),
        _ => &[],
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn primitive_identity() -> IdentityRule {
        IdentityRule::PrimitiveValue
    }

    fn key_identity() -> IdentityRule {
        IdentityRule::Key {
            key: "id".to_string(),
        }
    }

    #[test]
    fn test_ordered_unique_base_example() {
        // From spec: base=[A], current=[A,B], proposed=[A,C] → [A,B,C]
        let base = json!([{"id": "A"}]);
        let current = json!([{"id": "A"}, {"id": "B"}]);
        let proposed = json!([{"id": "A"}, {"id": "C"}]);

        let result = merge_ordered_unique("items", &base, &current, &proposed, &key_identity());
        assert!(result.is_merged());

        let merged = result.unwrap_merged();
        let arr = merged.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].get("id"), Some(&json!("A")));
        assert_eq!(arr[1].get("id"), Some(&json!("B")));
        assert_eq!(arr[2].get("id"), Some(&json!("C")));
    }

    #[test]
    fn test_ordered_unique_primitive() {
        let base = json!(["a", "b"]);
        let current = json!(["a", "b", "c"]);
        let proposed = json!(["a", "b", "d"]);

        let result =
            merge_ordered_unique("tags", &base, &current, &proposed, &primitive_identity());
        assert!(result.is_merged());

        let merged = result.unwrap_merged();
        let arr = merged.as_array().unwrap();
        assert_eq!(arr, &[json!("a"), json!("b"), json!("c"), json!("d")]);
    }

    #[test]
    fn test_ordered_unique_deduplicates() {
        let base = json!(["a"]);
        let current = json!(["a", "b", "b"]); // duplicate b
        let proposed = json!(["a", "c"]);

        let result =
            merge_ordered_unique("tags", &base, &current, &proposed, &primitive_identity());
        assert!(result.is_merged());

        let merged = result.unwrap_merged();
        let arr = merged.as_array().unwrap();
        assert_eq!(arr, &[json!("a"), json!("b"), json!("c")]);
    }

    #[test]
    fn test_ordered_unique_conflict_same_id_different_values() {
        let base = json!([{"id": "X", "val": 1}]);
        let current = json!([{"id": "X", "val": 2}]);
        let proposed = json!([{"id": "X", "val": 3}]);

        let result = merge_ordered_unique("items", &base, &current, &proposed, &key_identity());
        assert!(result.is_conflict());
    }

    #[test]
    fn test_ordered_unique_no_conflict_one_side_unchanged() {
        let base = json!([{"id": "X", "val": 1}]);
        let current = json!([{"id": "X", "val": 2}]);
        let proposed = json!([{"id": "X", "val": 1}]); // same as base

        let result = merge_ordered_unique("items", &base, &current, &proposed, &key_identity());
        assert!(result.is_merged());
        let merged = result.unwrap_merged();
        assert_eq!(merged[0].get("val"), Some(&json!(2)));
    }

    #[test]
    fn test_ordered_unique_both_add_same_no_conflict() {
        let base = json!([{"id": "A"}]);
        let current = json!([{"id": "A"}, {"id": "B"}]);
        let proposed = json!([{"id": "A"}, {"id": "B"}]); // both add B

        let result = merge_ordered_unique("items", &base, &current, &proposed, &key_identity());
        assert!(result.is_merged());
        let merged = result.unwrap_merged();
        assert_eq!(merged.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_ordered_unique_preserves_base_order() {
        let base = json!([{"id": "A"}, {"id": "B"}]);
        let current = json!([{"id": "A"}, {"id": "B"}, {"id": "C"}]);
        let proposed = json!([{"id": "B"}, {"id": "A"}, {"id": "D"}]);

        let result = merge_ordered_unique("items", &base, &current, &proposed, &key_identity());
        assert!(result.is_merged());
        let merged = result.unwrap_merged();
        let arr = merged.as_array().unwrap();
        // Base order: A, B preserved. Then C, D appended.
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].get("id"), Some(&json!("A")));
        assert_eq!(arr[1].get("id"), Some(&json!("B")));
        // C and D are in order of current, then proposed
    }

    #[test]
    fn test_ordered_unique_empty_arrays() {
        let base = json!([]);
        let current = json!([]);
        let proposed = json!([]);

        let result = merge_ordered_unique("items", &base, &current, &proposed, &key_identity());
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!([]));
    }

    #[test]
    fn test_ordered_unique_current_adds_proposed_empty() {
        let base = json!(["a"]);
        let current = json!(["a", "b"]);
        let proposed = json!(["a"]);

        let result =
            merge_ordered_unique("tags", &base, &current, &proposed, &primitive_identity());
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!(["a", "b"]));
    }
}
