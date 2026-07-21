//! Conflict representation for three-way merge outcomes.
//!
//! Conflicts are **structured** — no textual markers, no merge
//! syntax, no YAML corruption.  Every conflict carries the original
//! base, current, and proposed values so that agents and UIs can
//! resolve them programmatically.

use serde::Serialize;
use serde_json::Value;

/// Outcome of a three-way merge operation.
#[derive(Debug, Clone, Serialize)]
pub enum MergeResult {
    /// Merge succeeded.  Contains the merged document.
    Merged(Value),

    /// Merge produced conflicts that must be resolved before writing.
    Conflicts(Vec<Conflict>),
}

impl MergeResult {
    /// Returns `true` if this is a successful merge.
    pub fn is_merged(&self) -> bool {
        matches!(self, MergeResult::Merged(_))
    }

    /// Returns `true` if conflicts were produced.
    pub fn is_conflict(&self) -> bool {
        matches!(self, MergeResult::Conflicts(_))
    }

    /// Unwrap the merged value.  Panics if this is a conflict result.
    pub fn unwrap_merged(self) -> Value {
        match self {
            MergeResult::Merged(v) => v,
            MergeResult::Conflicts(c) => {
                panic!("called unwrap_merged on conflict result: {:?}", c)
            }
        }
    }

    /// Unwrap the conflicts vector.  Panics if this is a merged result.
    pub fn unwrap_conflicts(self) -> Vec<Conflict> {
        match self {
            MergeResult::Conflicts(c) => c,
            MergeResult::Merged(_) => {
                panic!("called unwrap_conflicts on merged result")
            }
        }
    }
}

impl From<Value> for MergeResult {
    fn from(v: Value) -> Self {
        MergeResult::Merged(v)
    }
}

// ── Conflict ────────────────────────────────────────────────────────────

/// A structured conflict from a three-way merge.
///
/// Each conflict records the exact path and the three values involved.
/// This is machine-readable by design — agents and UIs inspect these
/// to present resolution options.
#[derive(Debug, Clone, Serialize)]
pub struct Conflict {
    /// The canonical path to the conflicting value.
    /// e.g. `"root.properties.homeworld"` or `"root.characters[obiwan].name"`.
    pub path: String,

    /// The type of conflict.
    pub conflict_type: ConflictType,

    /// The value in the base (reference) document.
    pub base: Value,

    /// The value in the current (our) document.
    pub current: Value,

    /// The value in the proposed (their) document.
    pub proposed: Value,
}

// ── Conflict Type ──────────────────────────────────────────────────────

/// Classifies the nature of a merge conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    /// The values differ and no strategy can resolve the divergence.
    ValueMismatch,

    /// The types differ between current and proposed (e.g. string vs object).
    TypeMismatch,

    /// The structural shape differs (e.g. object vs array).
    StructuralConflict,

    /// An identity field was mutated, which is not allowed.
    IdentityMutation,
}

// ── Convenience ────────────────────────────────────────────────────────

impl Conflict {
    /// Create a new value-mismatch conflict.
    pub fn value_mismatch(
        path: impl Into<String>,
        base: Value,
        current: Value,
        proposed: Value,
    ) -> Self {
        Self {
            path: path.into(),
            conflict_type: ConflictType::ValueMismatch,
            base,
            current,
            proposed,
        }
    }

    /// Create a new type-mismatch conflict.
    pub fn type_mismatch(
        path: impl Into<String>,
        base: Value,
        current: Value,
        proposed: Value,
    ) -> Self {
        Self {
            path: path.into(),
            conflict_type: ConflictType::TypeMismatch,
            base,
            current,
            proposed,
        }
    }

    /// Create a new structural conflict.
    pub fn structural(
        path: impl Into<String>,
        base: Value,
        current: Value,
        proposed: Value,
    ) -> Self {
        Self {
            path: path.into(),
            conflict_type: ConflictType::StructuralConflict,
            base,
            current,
            proposed,
        }
    }

    /// Create a new identity mutation conflict.
    pub fn identity_mutation(
        path: impl Into<String>,
        base: Value,
        current: Value,
        proposed: Value,
    ) -> Self {
        Self {
            path: path.into(),
            conflict_type: ConflictType::IdentityMutation,
            base,
            current,
            proposed,
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_value_mismatch_conflict() {
        let c = Conflict::value_mismatch(
            "root.properties.homeworld",
            json!("Stewjon"),
            json!("Tatooine"),
            json!("Coruscant"),
        );
        assert_eq!(c.path, "root.properties.homeworld");
        assert!(matches!(c.conflict_type, ConflictType::ValueMismatch));
        assert_eq!(c.base, json!("Stewjon"));
        assert_eq!(c.current, json!("Tatooine"));
        assert_eq!(c.proposed, json!("Coruscant"));
    }

    #[test]
    fn test_merge_result_merged() {
        let v = json!({"name": "Luke"});
        let r = MergeResult::Merged(v.clone());
        assert!(r.is_merged());
        assert!(!r.is_conflict());
        assert_eq!(r.unwrap_merged(), v);
    }

    #[test]
    #[should_panic(expected = "unwrap_merged")]
    fn test_unwrap_merged_on_conflict_panics() {
        let r = MergeResult::Conflicts(vec![]);
        r.unwrap_merged();
    }

    #[test]
    fn test_merge_result_conflicts() {
        let c = Conflict::value_mismatch("x", json!("a"), json!("b"), json!("c"));
        let r = MergeResult::Conflicts(vec![c]);
        assert!(r.is_conflict());
        assert!(!r.is_merged());
        assert_eq!(r.unwrap_conflicts().len(), 1);
    }

    #[test]
    fn test_from_value() {
        let v = json!({"key": "value"});
        let r: MergeResult = v.clone().into();
        assert!(r.is_merged());
    }

    #[test]
    fn test_identity_mutation_conflict() {
        let c = Conflict::identity_mutation(
            "root.characters[obiwan].id",
            json!("obiwan"),
            json!("ben_kenobi"),
            json!("obiwan"),
        );
        assert!(matches!(c.conflict_type, ConflictType::IdentityMutation));
    }

    #[test]
    fn test_serialize_conflict() {
        let c = Conflict::value_mismatch(
            "root.name",
            json!("Old"),
            json!("Current"),
            json!("Proposed"),
        );
        let json_str = serde_json::to_string(&c).unwrap();
        assert!(json_str.contains("root.name"));
        assert!(json_str.contains("Current"));
        assert!(json_str.contains("Proposed"));
    }
}
