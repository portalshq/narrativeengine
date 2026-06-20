//! Validation layer for NAP structured merge.
//!
//! Three stages:
//!
//! **Stage 1 — SDL validation**
//! Ensure the SDL document itself is valid:
//! - Valid merge strategy for each property
//! - Required fields exist
//! - Identity keys defined where required
//! - Property names don't contain dots (V1 limitation)
//!
//! **Stage 2 — Manifest validation**
//! Ensure the manifest conforms to the SDL:
//! - Required fields are present
//! - Types match the schema
//!
//! **Stage 3 — Merge strategy validation**
//! Ensure merge strategies have all required metadata:
//! - `ordered_unique` has identity rule
//! - `edge_list` has source_key, target_key, identity rule
//!
//! All validation MUST complete before persistence.

pub mod manifest;
pub mod merge;
pub mod sdl;

use crate::merge::conflict::MergeResult;
use crate::merge::sdl::SdlDocument;
use serde_json::Value;

/// A validation error.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Human-readable description of the error.
    pub message: String,

    /// Optional path to the field that caused the error.
    pub path: Option<String>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.path {
            Some(p) => write!(f, "{}: {}", p, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

/// Result type for validation operations.
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// Run the full validation pipeline on a merge result.
///
/// Stages:
/// 1. SDL validation (call `validate_sdl` separately, before merge)
/// 2. Manifest validation (call `validate_manifest` separately, before merge)
/// 3. Post-merge validation — validates the merge result against the SDL
///
/// This is the "before persistence" gate.
pub fn validate_before_persist(schema: &SdlDocument, merged: &MergeResult) -> ValidationResult {
    match merged {
        MergeResult::Conflicts(_) => Err(vec![ValidationError {
            message: "cannot persist — merge produced unresolved conflicts".to_string(),
            path: None,
        }]),
        MergeResult::Merged(value) => merge::validate_merged(schema, value),
    }
}

/// Check that all required paths exist in a value.
pub fn check_required_paths(schema: &SdlDocument, value: &Value) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for required_path in &schema.schema.required {
        // Parse the path and check existence
        match crate::merge::path::CanonicalPath::parse(required_path) {
            Ok(path) => {
                if crate::merge::path::resolve_path(value, &path).is_none() {
                    errors.push(ValidationError {
                        message: format!("required field '{}' is missing", required_path),
                        path: Some(required_path.clone()),
                    });
                }
            }
            Err(_) => {
                errors.push(ValidationError {
                    message: format!("invalid required path '{}'", required_path),
                    path: Some(required_path.clone()),
                });
            }
        }
    }

    errors
}

/// Check that a value's type matches the SDL property type.
pub fn check_type_match(
    type_: &crate::merge::sdl::PropertyType,
    value: &Value,
    path: &str,
) -> Option<ValidationError> {
    // null is technically valid for any type (it's "unset")
    if value.is_null() {
        return None;
    }

    let type_matches = match type_ {
        crate::merge::sdl::PropertyType::String => value.is_string(),
        crate::merge::sdl::PropertyType::Number => value.is_number(),
        crate::merge::sdl::PropertyType::Boolean => value.is_boolean(),
        crate::merge::sdl::PropertyType::Object => value.is_object(),
        crate::merge::sdl::PropertyType::Array => value.is_array(),
    };

    if !type_matches {
        Some(ValidationError {
            message: format!(
                "type mismatch: expected {:?}, got {}",
                type_,
                type_name(value)
            ),
            path: Some(path.to_string()),
        })
    } else {
        None
    }
}

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

impl ValidationError {
    pub fn new(message: impl Into<String>, path: impl Into<String>) -> Self {
        ValidationError {
            message: message.into(),
            path: Some(path.into()),
        }
    }

    pub fn global(message: impl Into<String>) -> Self {
        ValidationError {
            message: message.into(),
            path: None,
        }
    }
}

/// Combine multiple validation results.
pub fn combine(results: Vec<ValidationResult>) -> ValidationResult {
    let mut all_errors = Vec::new();
    for result in results {
        match result {
            Ok(()) => {}
            Err(errors) => all_errors.extend(errors),
        }
    }
    if all_errors.is_empty() {
        Ok(())
    } else {
        Err(all_errors)
    }
}
