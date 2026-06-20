//! Pre-diff normalization engine.
//!
//! # Why normalize?
//!
//! Before generating diffs, the merge engine fills "missing" paths in
//! the candidate document from the base document.  This ensures that
//! omission (not having an opinion about a field) is not misinterpreted
//! as deletion.
//!
//! # Rule
//!
//! - **missing** = no change → copy from base
//! - **null** = explicit deletion → leave `null`
//! - **present** = modified → leave unchanged
//!
//! This is a **protocol invariant** — it is not configurable, not
//! expressed in SDL, and must never vary across implementations.

use serde_json::Value;

/// Normalize a candidate document against a base document.
///
/// For every leaf path in `base`:
/// - If the path is missing in `candidate`, copy the base value.
/// - If the path exists as `null`, leave `null` (explicit deletion).
/// - If the path exists with a value, leave unchanged.
///
/// Paths in `candidate` that do not exist in `base` are preserved
/// (they represent additions).
///
/// # Returns
///
/// A new `Value` representing the normalized candidate.
pub fn normalize(base: &Value, candidate: &Value) -> Value {
    // Walk both trees simultaneously and produce a merged result.
    merge_missing(base, candidate)
}

/// Recursively merge `base` into `candidate` for missing paths only.
fn merge_missing(base: &Value, candidate: &Value) -> Value {
    match (base, candidate) {
        // Both are objects — recurse into shared keys
        (Value::Object(base_map), Value::Object(candidate_map)) => {
            let mut result = candidate_map.clone();

            for (key, base_val) in base_map {
                match result.get(key) {
                    // Path missing in candidate → copy from base
                    None => {
                        result.insert(key.clone(), base_val.clone());
                    }
                    // Path exists in candidate → recurse deeper
                    Some(candidate_val) => {
                        let merged = merge_missing(base_val, candidate_val);
                        // Only update if the merge actually changed something
                        // (avoids unnecessary cloning of identical values)
                        if &merged != candidate_val {
                            result.insert(key.clone(), merged);
                        }
                    }
                }
            }

            Value::Object(result)
        }

        // Base is object, candidate is missing/deeper structure mismatch
        // → candidate wins (it's not an object, so we can't recurse)
        (Value::Object(_), _) => candidate.clone(),

        // At least one is a non-object → leaf, no further recursion needed
        // Candidate value stands (whether it matches base or differs)
        _ => candidate.clone(),
    }
}

/// Check if a value is the JSON `null` literal.
pub fn is_explicit_null(value: &Value) -> bool {
    value.is_null()
}

/// Extract a value from an optional JSON value reference.
/// Returns `None` for missing values and `Some(Value::Null)` for explicit nulls.
pub fn extract_optional(value: Option<&Value>) -> OptionalValue {
    match value {
        None => OptionalValue::Missing,
        Some(v) if v.is_null() => OptionalValue::Null,
        Some(v) => OptionalValue::Present(v.clone()),
    }
}

/// Represents the presence semantics of a value.
#[derive(Debug, Clone, PartialEq)]
pub enum OptionalValue {
    /// Path does not exist in the document.
    Missing,
    /// Path explicitly set to null (deletion intent).
    Null,
    /// Path has an actual value.
    Present(Value),
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_missing_field_copied_from_base() {
        let base = json!({"name": "Obi Wan", "homeworld": "Stewjon"});
        let candidate = json!({"name": "Obi Wan Kenobi"});

        let result = normalize(&base, &candidate);

        assert_eq!(result.get("name"), Some(&json!("Obi Wan Kenobi")));
        assert_eq!(result.get("homeworld"), Some(&json!("Stewjon")));
    }

    #[test]
    fn test_explicit_null_preserved() {
        let base = json!({"homeworld": "Stewjon"});
        let candidate = json!({"homeworld": null});

        let result = normalize(&base, &candidate);

        assert_eq!(result.get("homeworld"), Some(&Value::Null));
    }

    #[test]
    fn test_present_field_unchanged() {
        let base = json!({"name": "Luke", "species": "human"});
        let candidate = json!({"name": "Luke Skywalker", "species": "human"});

        let result = normalize(&base, &candidate);

        assert_eq!(result.get("name"), Some(&json!("Luke Skywalker")));
        assert_eq!(result.get("species"), Some(&json!("human")));
    }

    #[test]
    fn test_additions_are_preserved() {
        let base = json!({"name": "Luke"});
        let candidate = json!({"name": "Luke", "homeworld": "Tatooine"});

        let result = normalize(&base, &candidate);

        assert_eq!(result.get("name"), Some(&json!("Luke")));
        assert_eq!(result.get("homeworld"), Some(&json!("Tatooine")));
    }

    #[test]
    fn test_nested_missing_filled() {
        let base = json!({
            "character": {
                "name": "Obi Wan",
                "homeworld": "Stewjon"
            }
        });
        let candidate = json!({
            "character": {
                "name": "Obi Wan Kenobi"
            }
        });

        let result = normalize(&base, &candidate);

        assert_eq!(
            result.pointer("/character/name"),
            Some(&json!("Obi Wan Kenobi"))
        );
        assert_eq!(
            result.pointer("/character/homeworld"),
            Some(&json!("Stewjon"))
        );
    }

    #[test]
    fn test_nested_null_preserved() {
        let base = json!({
            "character": {
                "homeworld": "Stewjon",
                "species": "human"
            }
        });
        let candidate = json!({
            "character": {
                "homeworld": null,
                "species": "human"
            }
        });

        let result = normalize(&base, &candidate);

        assert_eq!(result.pointer("/character/homeworld"), Some(&Value::Null));
        assert_eq!(result.pointer("/character/species"), Some(&json!("human")));
    }

    #[test]
    fn test_all_equal_no_change() {
        let base = json!({"a": 1, "b": 2});
        let candidate = json!({"a": 1, "b": 2});

        let result = normalize(&base, &candidate);

        assert_eq!(result, candidate);
    }

    #[test]
    fn test_candidate_wins_for_new_paths() {
        let base = json!({"a": 1});
        let candidate = json!({"a": 1, "b": 2, "c": 3});

        let result = normalize(&base, &candidate);

        assert_eq!(result, candidate);
    }

    #[test]
    fn test_empty_base_returns_candidate_unchanged() {
        let base = json!({});
        let candidate = json!({"a": 1, "b": 2});

        let result = normalize(&base, &candidate);

        assert_eq!(result, candidate);
    }

    #[test]
    fn test_type_mismatch_candidate_wins() {
        // If base has an object but candidate has a string at the same path,
        // candidate wins (can't recurse into different types).
        let base = json!({"nested": {"key": "value"}});
        let candidate = json!({"nested": "replaced"});

        let result = normalize(&base, &candidate);

        assert_eq!(result, candidate);
    }

    #[test]
    fn test_optional_value_semantics() {
        assert_eq!(extract_optional(None), OptionalValue::Missing);
        assert_eq!(extract_optional(Some(&Value::Null)), OptionalValue::Null);
        assert_eq!(
            extract_optional(Some(&json!("hello"))),
            OptionalValue::Present(json!("hello"))
        );
        assert!(is_explicit_null(&Value::Null));
        assert!(!is_explicit_null(&json!("hello")));
    }
}
