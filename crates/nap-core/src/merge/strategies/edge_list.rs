//! Edge list merge strategy.
//!
//! For graph relationship lists where each edge has a source, target,
//! and unique identity.
//!
//! - New edges added independently by current and proposed merge automatically.
//! - Conflict only when the same edge identity is modified differently.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::merge::conflict::{Conflict, MergeResult};
use crate::merge::sdl::IdentityRule;

/// Edge-list merge for an array of edge objects.
///
/// Each edge has:
/// - `identity_key`: unique edge identifier
/// - `source_key`: reference to source node
/// - `target_key`: reference to target node
///
/// # Algorithm
///
/// 1. Index by identity.
/// 2. If same identity has different source/target in current vs proposed,
///    and neither matches base → conflict.
/// 3. Otherwise take the union, preferring non-base values.
///
/// # Arguments
///
/// * `path` - Canonical path for diagnostics.
/// * `base` - Base array of edge objects.
/// * `current` - Current array of edge objects.
/// * `proposed` - Proposed array of edge objects.
/// * `identity` - Identity rule (must be Key mode).
/// * `source_key` - The key within each edge object for the source reference.
/// * `target_key` - The key within each edge object for the target reference.
pub fn merge_edge_list(
    path: &str,
    base: &Value,
    current: &Value,
    proposed: &Value,
    identity: &IdentityRule,
    _source_key: &str,
    _target_key: &str,
) -> MergeResult {
    let base_arr = as_array(base);
    let current_arr = as_array(current);
    let proposed_arr = as_array(proposed);

    let base_by_id = index_by_identity(base_arr, identity);
    let current_by_id = index_by_identity(current_arr, identity);
    let proposed_by_id = index_by_identity(proposed_arr, identity);

    // Check for conflicts: same edge identity modified differently
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

    // Build merged edge set
    let mut by_id: BTreeMap<String, Value> = BTreeMap::new();

    // Start with base
    for (id, val) in &base_by_id {
        by_id.insert(id.clone(), val.clone());
    }

    // Apply current modifications
    for (id, val) in &current_by_id {
        by_id.insert(id.clone(), val.clone());
    }

    // Apply proposed modifications (only where current didn't)
    for (id, val) in &proposed_by_id {
        if !current_by_id.contains_key(id) {
            by_id.insert(id.clone(), val.clone());
        }
    }

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

    fn key_identity() -> IdentityRule {
        IdentityRule::Key {
            key: "id".to_string(),
        }
    }

    #[test]
    fn test_edge_list_independent_adds_merge() {
        let base = json!([{"id": "e1", "character": "luke", "scene": "cantina"}]);
        let current = json!([{"id": "e1", "character": "luke", "scene": "cantina"}, {"id": "e2", "character": "leia", "scene": "death-star"}]);
        let proposed = json!([{"id": "e1", "character": "luke", "scene": "cantina"}, {"id": "e3", "character": "han", "scene": "cantina"}]);

        let result = merge_edge_list(
            "edges",
            &base,
            &current,
            &proposed,
            &key_identity(),
            "character",
            "scene",
        );
        assert!(result.is_merged());

        let merged = result.unwrap_merged();
        let arr = merged.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_edge_list_conflict_on_same_edge() {
        let base = json!([{"id": "e1", "character": "luke", "scene": "cantina"}]);
        let current = json!([{"id": "e1", "character": "luke", "scene": "mos-eisley"}]);
        let proposed = json!([{"id": "e1", "character": "luke", "scene": "jedi-temple"}]);

        let result = merge_edge_list(
            "edges",
            &base,
            &current,
            &proposed,
            &key_identity(),
            "character",
            "scene",
        );
        assert!(result.is_conflict());
    }

    #[test]
    fn test_edge_list_one_side_unchanged() {
        let base = json!([{"id": "e1", "character": "luke", "scene": "cantina"}]);
        let current = json!([{"id": "e1", "character": "luke", "scene": "tatooine"}]);
        let proposed = json!([{"id": "e1", "character": "luke", "scene": "cantina"}]); // unchanged from base

        let result = merge_edge_list(
            "edges",
            &base,
            &current,
            &proposed,
            &key_identity(),
            "character",
            "scene",
        );
        assert!(result.is_merged());

        let merged = result.unwrap_merged();
        assert_eq!(merged[0].get("scene"), Some(&json!("tatooine")));
    }

    #[test]
    fn test_edge_list_empty_base() {
        let base = json!([]);
        let current = json!([{"id": "e1", "character": "luke", "scene": "cantina"}]);
        let proposed = json!([{"id": "e2", "character": "vader", "scene": "death-star"}]);

        let result = merge_edge_list(
            "edges",
            &base,
            &current,
            &proposed,
            &key_identity(),
            "character",
            "scene",
        );
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged().as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_edge_list_all_empty() {
        let base = json!([]);
        let current = json!([]);
        let proposed = json!([]);

        let result = merge_edge_list(
            "edges",
            &base,
            &current,
            &proposed,
            &key_identity(),
            "character",
            "scene",
        );
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!([]));
    }

    #[test]
    fn test_edge_list_both_add_same_edge_no_conflict() {
        let base = json!([{"id": "e1", "character": "luke", "scene": "cantina"}]);
        let current = json!([{"id": "e1", "character": "luke", "scene": "cantina"}, {"id": "e2", "character": "han", "scene": "falcon"}]);
        let proposed = json!([{"id": "e1", "character": "luke", "scene": "cantina"}, {"id": "e2", "character": "han", "scene": "falcon"}]);

        let result = merge_edge_list(
            "edges",
            &base,
            &current,
            &proposed,
            &key_identity(),
            "character",
            "scene",
        );
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged().as_array().unwrap().len(), 2);
    }
}
