//! Replace merge strategy.
//!
//! Simple value overwrite using the Rule 6 conflict matrix.
//! No recursion, no special identity handling.

use serde_json::Value;

use crate::merge::conflict::{Conflict, MergeResult};

/// Merge a single value using the replace strategy.
///
/// Applies the conflict matrix:
/// - `current == base && proposed != base` → accept proposed
/// - `proposed == base && current != base` → accept current
/// - `current == proposed` → accept either
/// - all equal → accept base
/// - `current != base && proposed != base && current != proposed` → conflict
pub fn merge_replace(path: &str, base: &Value, current: &Value, proposed: &Value) -> MergeResult {
    // All equal → accept base
    if current == base && proposed == base {
        return MergeResult::Merged(base.clone());
    }

    // Only current changed → accept current
    if current != base && proposed == base {
        return MergeResult::Merged(current.clone());
    }

    // Only proposed changed → accept proposed
    if current == base && proposed != base {
        return MergeResult::Merged(proposed.clone());
    }

    // Both changed to the same value → accept either
    if current == proposed {
        return MergeResult::Merged(current.clone());
    }

    // Both changed differently → conflict
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
    fn test_replace_accept_current() {
        let result = merge_replace(
            "name",
            &json!("Luke"),
            &json!("Luke Skywalker"),
            &json!("Luke"),
        );
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!("Luke Skywalker"));
    }

    #[test]
    fn test_replace_accept_proposed() {
        let result = merge_replace(
            "name",
            &json!("Luke"),
            &json!("Luke"),
            &json!("Luke Skywalker"),
        );
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!("Luke Skywalker"));
    }

    #[test]
    fn test_replace_accept_either_when_equal() {
        let result = merge_replace(
            "name",
            &json!("Luke"),
            &json!("Luke Skywalker"),
            &json!("Luke Skywalker"),
        );
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!("Luke Skywalker"));
    }

    #[test]
    fn test_replace_accept_all_equal() {
        let result = merge_replace("name", &json!("Luke"), &json!("Luke"), &json!("Luke"));
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!("Luke"));
    }

    #[test]
    fn test_replace_conflict() {
        let result = merge_replace(
            "name",
            &json!("Luke"),
            &json!("Luke Skywalker"),
            &json!("Anakin"),
        );
        assert!(result.is_conflict());
        let conflicts = result.unwrap_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].current, json!("Luke Skywalker"));
        assert_eq!(conflicts[0].proposed, json!("Anakin"));
    }

    #[test]
    fn test_replace_with_numbers() {
        let result = merge_replace("version", &json!(1), &json!(2), &json!(1));
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!(2));
    }

    #[test]
    fn test_replace_with_null() {
        // Explicit deletion vs change
        let result = merge_replace(
            "homeworld",
            &json!("Stewjon"),
            &json!(Value::Null),
            &json!("Stewjon"),
        );
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), Value::Null);
    }
}
