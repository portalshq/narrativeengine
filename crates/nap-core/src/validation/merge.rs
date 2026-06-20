//! Post-merge validation (Stage 3).
//!
//! Validates the merged result against the SDL before persistence:
//!
//! - All required paths exist.
//! - Field types match the schema.
//! - Array items have required identity keys.
//!
//! This is the final gate before atomic write.

use serde_json::Value;

use crate::merge::path::{CanonicalPath, build_path_map};
use crate::merge::sdl::{PropertyType, SdlDocument};
use crate::validation::{
    ValidationError, ValidationResult, check_required_paths, check_type_match,
};

/// Validate a merged document before persistence.
///
/// Returns `Ok(())` if the merged document passes all SDL checks.
///
/// # Errors
///
/// Returns a list of `ValidationError` describing every violation.
pub fn validate_merged(schema: &SdlDocument, merged: &Value) -> ValidationResult {
    let mut errors = Vec::new();

    // 1. Check required paths
    errors.extend(check_required_paths(schema, merged));

    // 2. Build a path map for type checking each defined property
    let path_map = build_path_map(merged, schema);

    for (path, def) in &schema.schema.properties {
        // Try to find the value at this path
        let value = path_map.get(path).or_else(|| {
            // Fallback: try direct path resolution
            CanonicalPath::parse(path)
                .ok()
                .and_then(|cp| crate::merge::path::resolve_path(merged, &cp))
        });

        if let Some(val) = value {
            // Check type match
            if let Some(err) = check_type_match(&def.type_, val, path) {
                errors.push(err);
            }

            // For arrays with identity rules, verify items have identity key
            if matches!(def.type_, PropertyType::Array)
                && let Some(ref identity) = def.merge.identity
                && let Value::Array(arr) = val
                && let crate::merge::sdl::IdentityRule::Key { key } = identity
            {
                for (idx, item) in arr.iter().enumerate() {
                    if let Value::Object(obj) = item
                        && !obj.contains_key(key)
                    {
                        errors.push(ValidationError::new(
                            format!(
                                "merged item at index {} is missing identity key '{}'",
                                idx, key
                            ),
                            format!("{}[{}]", path, idx),
                        ));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merge::sdl::SdlDocument;
    use serde_json::json;

    fn test_sdl() -> SdlDocument {
        SdlDocument::from_yaml(
            r#"
schema:
  version: "1.0"
  required:
    - id
  properties:
    id:
      type: string
      merge:
        type: replace
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
"#,
        )
        .unwrap()
    }

    #[test]
    fn test_validate_merged_valid() {
        let merged = json!({
            "id": "nap://test/char/luke",
            "name": "Luke Skywalker",
            "version": 2,
            "tags": ["hero", "jedi"],
            "characters": [
                {"id": "obiwan", "name": "Obi-Wan"}
            ]
        });

        assert!(validate_merged(&test_sdl(), &merged).is_ok());
    }

    #[test]
    fn test_validate_merged_missing_required() {
        let merged = json!({
            "name": "Luke"
            // missing "id"
        });

        let result = validate_merged(&test_sdl(), &merged);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_merged_type_mismatch() {
        let merged = json!({
            "id": "nap://test/char/luke",
            "version": "not-a-number"
        });

        let result = validate_merged(&test_sdl(), &merged);
        assert!(result.is_err());
    }
}
