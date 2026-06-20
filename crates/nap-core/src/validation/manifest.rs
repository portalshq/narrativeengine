//! Manifest validation against SDL (Stage 2).
//!
//! Validates that a manifest conforms to its SDL schema:
//!
//! - Required fields are present.
//! - Field types match the schema.
//! - Identity keys are present for array items that use identity rules.

use serde_json::Value;

use crate::merge::path::{CanonicalPath, build_path_map};
use crate::merge::sdl::{IdentityRule, PropertyType, SdlDocument};
use crate::validation::{
    ValidationError, ValidationResult, check_required_paths, check_type_match,
};

/// Validate a manifest document against its SDL schema.
///
/// Checks:
/// 1. All required paths exist.
/// 2. Each defined property matches its declared type.
/// 3. Array items with identity rules have the identity key.
///
/// # Arguments
///
/// * `schema` - The SDL document.
/// * `manifest` - The manifest to validate (as `serde_json::Value`).
pub fn validate_manifest(schema: &SdlDocument, manifest: &Value) -> ValidationResult {
    let mut errors = Vec::new();

    // 1. Check required paths
    errors.extend(check_required_paths(schema, manifest));

    // 2. Check property types
    for (path, def) in &schema.schema.properties {
        // Parse path and resolve value
        match CanonicalPath::parse(path) {
            Ok(canonical_path) => {
                let value = crate::merge::path::resolve_path(manifest, &canonical_path);
                if let Some(val) = value {
                    // Check type match
                    if let Some(err) = check_type_match(&def.type_, val, path) {
                        errors.push(err);
                    }

                    // 3. For arrays with identity rules, check that items have the identity key
                    if matches!(def.type_, PropertyType::Array)
                        && let Some(ref identity) = def.merge.identity
                    {
                        errors.extend(check_array_identity(val, path, identity));
                    }
                }
            }
            Err(_) => {
                // Path might be a deep path like "properties.homeworld"
                // which is handled differently — use build_path_map
                let path_map = build_path_map(manifest, schema);
                if let Some(val) = path_map.get(path)
                    && let Some(err) = check_type_match(&def.type_, val, path)
                {
                    errors.push(err);
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

/// Check that array items have the required identity key.
fn check_array_identity(
    value: &Value,
    path: &str,
    identity: &IdentityRule,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    match identity {
        IdentityRule::PrimitiveValue => {
            // No key to check — identity is the value itself
        }
        IdentityRule::Key { key } => {
            if let Value::Array(arr) = value {
                for (idx, item) in arr.iter().enumerate() {
                    if let Value::Object(obj) = item {
                        if !obj.contains_key(key) {
                            errors.push(ValidationError::new(
                                format!(
                                    "array item at index {} is missing identity key '{}'",
                                    idx, key
                                ),
                                format!("{}[{}]", path, idx),
                            ));
                        }
                    } else if !item.is_null() {
                        // Non-object items in a key-mode array
                        errors.push(ValidationError::new(
                            format!(
                                "array item at index {} is not an object (expected identity key '{}')",
                                idx, key
                            ),
                            format!("{}[{}]", path, idx),
                        ));
                    }
                }
            }
        }
    }

    errors
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
    fn test_validate_valid_manifest() {
        let manifest = json!({
            "id": "nap://test/char/luke",
            "name": "Luke Skywalker",
            "version": 1,
            "tags": ["hero", "jedi"],
            "characters": [
                {"id": "obiwan", "name": "Obi-Wan"}
            ]
        });

        assert!(validate_manifest(&test_sdl(), &manifest).is_ok());
    }

    #[test]
    fn test_validate_missing_required() {
        let manifest = json!({
            "name": "Luke"
            // missing "id"
        });

        let result = validate_manifest(&test_sdl(), &manifest);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("required")));
    }

    #[test]
    fn test_validate_type_mismatch() {
        let manifest = json!({
            "id": "nap://test/char/luke",
            "version": "not-a-number"  // should be number
        });

        let result = validate_manifest(&test_sdl(), &manifest);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("type mismatch")));
    }

    #[test]
    fn test_validate_missing_identity_key_in_array() {
        let manifest = json!({
            "id": "nap://test/char/luke",
            "characters": [
                {"name": "Obi-Wan"}  // missing "id" key
            ]
        });

        let result = validate_manifest(&test_sdl(), &manifest);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("identity key")));
    }
}
