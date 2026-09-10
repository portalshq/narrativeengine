//! Atomic merge strategy.
//!
//! The entire value is treated as a single atomic unit.
//! Any divergent modification between current and proposed creates a
//! conflict — there is no concept of "merging" sub-parts.

use serde_json::Value;

use crate::merge::conflict::{Conflict, MergeResult};

/// Merge a value using the atomic strategy.
///
/// - If both branches made the **same** change (or no change), accept it.
/// - If both branches made **different** changes, conflict.
pub fn merge_atomic(path: &str, base: &Value, current: &Value, proposed: &Value) -> MergeResult {
    // Both unchanged → accept base
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

    // Both made the same change → accept either
    if current == proposed {
        return MergeResult::Merged(current.clone());
    }

    // Different changes → conflict (even if they seem like they could merge)
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
    fn test_atomic_same_change_accepted() {
        let result = merge_atomic("version", &json!(1), &json!(2), &json!(2));
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!(2));
    }

    #[test]
    fn test_atomic_current_change_only() {
        let result = merge_atomic("version", &json!(1), &json!(2), &json!(1));
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!(2));
    }

    #[test]
    fn test_atomic_proposed_change_only() {
        let result = merge_atomic("version", &json!(1), &json!(1), &json!(3));
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!(3));
    }

    #[test]
    fn test_atomic_divergent_changes_conflict() {
        let result = merge_atomic("version", &json!(1), &json!(2), &json!(3));
        assert!(result.is_conflict());
    }

    #[test]
    fn test_atomic_object_divergent_conflict() {
        let result = merge_atomic(
            "config",
            &json!({"a": 1}),
            &json!({"a": 1, "b": 2}),
            &json!({"a": 1, "c": 3}),
        );
        // Even though both added non-overlapping keys, atomic means whole-value.
        assert!(result.is_conflict());
    }

    #[test]
    fn test_atomic_all_equal() {
        let result = merge_atomic("version", &json!(5), &json!(5), &json!(5));
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), json!(5));
    }

    #[test]
    fn test_atomic_null_values() {
        let result = merge_atomic(
            "field",
            &json!("value"),
            &json!(Value::Null),
            &json!("value"),
        );
        assert!(result.is_merged());
        assert_eq!(result.unwrap_merged(), Value::Null);
    }
}
