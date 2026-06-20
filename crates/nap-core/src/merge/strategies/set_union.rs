//! Set union merge strategy.
//!
//! For unordered sets of items where identity matters:
//! - Deduplicate by identity.
//! - Take the union of all items from base, current, and proposed.
//! - Order is not preserved (this is a SET, not a list).
//! - Conflicts when both branches modify the same identity differently.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::merge::conflict::{Conflict, MergeResult};
use crate::merge::sdl::IdentityRule;

/// Set-union merge for an array.
///
/// # Algorithm
///
/// 1. Index base, current, proposed by identity.
/// 2. Collect all unique identities.
/// 3. If any identity exists in current and proposed with different values
///    (and neither matches base), conflict.
/// 4. Otherwise, return the union, preferring non-base values.
///
/// # Arguments
///
/// * `path` - Canonical path for diagnostics.
/// * `base` - Base array.
/// * `current` - Current (our) array.
/// * `proposed` - Proposed (their) array.
/// * `identity` - Identity rule (primitive_value or key).
pub fn merge_set_union(
    path: &str,
    base: &Value,
    current: &Value,
    proposed: &Value,
    identity: &IdentityRule,
) -> MergeResult {
    let base_arr = as_array(base);
    let current_arr = as_array(current);
    let proposed_arr = as_array(proposed);

    let base_by_id = index_by_identity(base_arr, identity);
    let current_by_id = index_by_identity(current_arr, identity);
    let proposed_by_id = index_by_identity(proposed_arr, identity);

    // Check for identity conflicts
    let mut conflicts = Vec::new();

    for (id, cur_val) in &current_by_id {
        if let Some(pro_val) = proposed_by_id.get(id)
            && cur_val != pro_val
        {
            let base_val = base_by_id.get(id);
            let cur_matches_base = base_val == Some(cur_val);
            let pro_matches_base = base_val == Some(pro_val);

            if !cur_matches_base && !pro_matches_base {
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

    // Build union — for each identity, prefer the non-base value
    let mut by_id: BTreeMap<String, Value> = BTreeMap::new();

    // Start with base items
    for (id, val) in &base_by_id {
        by_id.insert(id.clone(), val.clone());
    }

    // Override with current items (non-base preferred)
    for (id, val) in &current_by_id {
        by_id.insert(id.clone(), val.clone());
    }

    // Override with proposed items (only if current didn't already take it)
    for (id, val) in &proposed_by_id {
        if !current_by_id.contains_key(id) {
            by_id.insert(id.clone(), val.clone());
        }
    }

    // Collect into array (order is NOT guaranteed — this is a set)
    let result: Vec<Value> = by_id.into_values().collect();

    MergeResult::Merged(Value::Array(result))
}

// ── Helpers ────────────────────────────────────────────────────────────

fn extract_identity(val: &Value, identity: &IdentityRule) -> String {
    match identity {
        IdentityRule::PrimitiveValue => match val {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => serde_json::to_string(val).unwrap_or_else(|_| "unknown".to_string()),
        },
        IdentityRule::Key { key } => val
            .get(key)
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                _ => None,
            })
            .unwrap_or_else(|| "unknown".to_string()),
    }
}

fn index_by_identity(arr: &[Value], identity: &IdentityRule) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    for val in arr {
        let id = extract_identity(val, identity);
        map.insert(id, val.clone());
    }
    map
}

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
    fn test_set_union_basic() {
        let base = json!(["a", "b"]);
        let current = json!(["a", "c"]);
        let proposed = json!(["b", "d"]);

        let result = merge_set_union("tags", &base, &current, &proposed, &primitive_identity());
        assert!(result.is_merged());

        let merged = result.unwrap_merged();
        let arr = merged.as_array().unwrap();
        let mut ids: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
        ids.sort();
        assert_eq!(ids, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_set_union_with_keys() {
        let base = json!([{"id": "A", "val": 1}]);
        let current = json!([{"id": "A", "val": 1}, {"id": "B", "val": 2}]);
        let proposed = json!([{"id": "A", "val": 1}, {"id": "C", "val": 3}]);

        let result = merge_set_union("items", &base, &current, &proposed, &key_identity());
        assert!(result.is_merged());

        let merged = result.unwrap_merged();
        let arr = merged.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_set_union_conflict() {
        let base = json!([{"id": "X", "val": 1}]);
        let current = json!([{"id": "X", "val": 2}]);
        let proposed = json!([{"id": "X", "val": 3}]);

        let result = merge_set_union("items", &base, &current, &proposed, &key_identity());
        assert!(result.is_conflict());
    }

    #[test]
    fn test_set_union_no_conflict_one_side_matches_base() {
        let base = json!(["a"]);
        let current = json!(["b"]);
        let proposed = json!(["a"]); // same as base

        let result = merge_set_union("tags", &base, &current, &proposed, &primitive_identity());
        assert!(result.is_merged());
    }

    #[test]
    fn test_set_union_deduplicates() {
        let base = json!(["a", "b"]);
        let current = json!(["a", "b", "c"]);
        let proposed = json!(["a", "b", "c", "d"]);

        let result = merge_set_union("tags", &base, &current, &proposed, &primitive_identity());
        assert!(result.is_merged());

        let merged = result.unwrap_merged();
        let arr = merged.as_array().unwrap();
        // Should have exactly a, b, c, d (deduplicated)
        assert_eq!(arr.len(), 4);
    }

    #[test]
    fn test_set_union_empty() {
        let base = json!([]);
        let current = json!([]);
        let proposed = json!([]);

        let result = merge_set_union("tags", &base, &current, &proposed, &primitive_identity());
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!([]));
    }

    #[test]
    fn test_set_union_all_same() {
        let base = json!(["a"]);
        let current = json!(["a"]);
        let proposed = json!(["a"]);

        let result = merge_set_union("tags", &base, &current, &proposed, &primitive_identity());
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!(["a"]));
    }
}
