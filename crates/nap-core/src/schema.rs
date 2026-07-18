//! JSON Schema generation for NAP core types.
//!
//! These schemas describe the NAP data model (manifest, commit, etc.)
//! in standard JSON Schema draft-2020-12 format.  They are served by the
//! HTTP resolver at `/schema/{name}` and consumed by the MCP server's
//! `nap_get_schema` tool so that agents can introspect the data model
//! without guessing field names or types.

use serde_json::Value;

/// Returns the JSON Schema for the full Manifest struct.
pub fn manifest_schema() -> Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "nap://schema/manifest.json",
        "title": "NAP Manifest",
        "description": concat!(
            "The durable representation of a narrative resource. ",
            "Human-editable, machine-readable, agent-queryable. ",
            "Characters, locations, scenes, props, and worlds ",
            "all share this common structure."
        ),
        "type": "object",
        "required": ["id", "name", "entity_type"],
        "properties": {
            "id": {
                "type": "string",
                "pattern": "^nap://[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+$",
                "description": concat!(
                    "Canonical NAP URI. ",
                    "e.g., nap://starwars/character/lukeskywalker"
                )
            },
            "name": {
                "type": "string",
                "description": "Human-readable display name. e.g., 'Luke Skywalker'"
            },
            "entity_type": {
                "type": "string",
                "minLength": 1,
                "description": "The kind of narrative entity this manifest describes (any non-empty string)"
            },
            "version": {
                "type": "integer",
                "minimum": 0,
                "description": "Monotonic version counter. Incremented on each commit."
            },
            "principals": {
                "type": "object",
                "description": "Access control owners/maintainers/publishers (optional).",
                "properties": {
                    "owners": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Full control — can modify, transfer, delete."
                    },
                    "maintainers": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Can modify content but not transfer ownership."
                    },
                    "publishers": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Can publish/distribute but not modify source."
                    }
                }
            },
            "properties": {
                "type": "object",
                "additionalProperties": true,
                "description": concat!(
                    "Entity-specific key-value properties. ",
                    "Character: species, homeworld, affiliation, lightsaber_color. ",
                    "Scene: setting, participants, mood, time_of_day, outcome. ",
                    "Location: climate, type, controlled_by, population. ",
                    "Prop: owner, material, weight, color."
                )
            },
            "representations": {
                "type": "object",
                "additionalProperties": {
                    "$ref": "#/definitions/Representation"
                },
                "description": concat!(
                    "Content-addressed representations of this entity. ",
                    "Keys are labels like 'reference_image', 'voice_model', 'mesh'."
                )
            },
            "references": {
                "type": "object",
                "additionalProperties": true,
                "description": concat!(
                    "Cross-references to other NAP resources. ",
                    "Common keys: appears_in (array of scene URIs), ",
                    "relationships (array of {target, type} objects), ",
                    "owner (character URI)."
                )
            },
            "provenance": {
                "$ref": "#/definitions/Provenance",
                "description": concat!(
                    "AI generation provenance metadata — which model, ",
                    "prompt, seed, and parameters were used."
                )
            },
            "head": {
                "type": "string",
                "pattern": "^[a-f0-9]{40}$|^[a-f0-9]{64}$",
                "description": concat!(
                    "Pointer to the latest VCS commit hash (40-char Git SHA-1 ",
                    "or 64-char BLAKE3). ",
                    "History lives in the VCS, not the manifest."
                )
            },
            "metadata": {
                "type": "object",
                "additionalProperties": true,
                "description": "Arbitrary extension metadata. Future-proof escape hatch."
            }
        },
        "definitions": {
            "Representation": {
                "type": "object",
                "required": ["hash", "format"],
                "properties": {
                    "hash": {
                        "type": "string",
                        "pattern": "^blake3:[a-f0-9]{64}$",
                        "description": concat!(
                            "BLAKE3 content hash of the asset. ",
                            "Format: blake3:<64-hex-chars>"
                        )
                    },
                    "format": {
                        "type": "string",
                        "description": concat!(
                            "File format. ",
                            "Examples: png, glb, onnx, wav, mp4, splat"
                        )
                    },
                    "uri": {
                        "type": "string",
                        "description": concat!(
                            "Optional storage URI. ",
                            "e.g., gs://assets/starwars/luke/ref.png, ",
                            "s3://bucket/path/to/file.glb"
                        )
                    },
                    "tier": {
                        "type": "string",
                        "enum": ["draft", "production", "distribution"],
                        "description": concat!(
                            "Quality tier of the representation. ",
                            "draft = WIP, production = final, distribution = optimized"
                        )
                    }
                }
            },
            "Provenance": {
                "type": "object",
                "properties": {
                    "model": {
                        "type": "string",
                        "description": concat!(
                            "AI model used to generate this entity. ",
                            "e.g., 'midjourney-v6', 'gpt-4o', 'stable-diffusion-3'"
                        )
                    },
                    "prompt_hash": {
                        "type": "string",
                        "pattern": "^blake3:[a-f0-9]{64}$",
                        "description": "BLAKE3 content hash of the generation prompt."
                    },
                    "seed": {
                        "type": "string",
                        "description": "Generation seed for reproducibility."
                    },
                    "parameters": {
                        "type": "object",
                        "additionalProperties": {
                            "type": "string"
                        },
                        "description": concat!(
                            "Additional generation parameters. ",
                            "e.g., stylize, chaos, temperature, cfg_scale"
                        )
                    },
                    "derived_from": {
                        "type": "string",
                        "pattern": "^nap://",
                        "description": concat!(
                            "Parent entity URI this was derived from. ",
                            "e.g., nap://starwars/character/lukeskywalker/v1"
                        )
                    },
                    "created_at": {
                        "type": "string",
                        "format": "date-time",
                        "description": "ISO 8601 creation timestamp."
                    }
                }
            }
        }
    })
}

/// Returns the JSON Schema for a NAP Commit.
pub fn commit_schema() -> Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "nap://schema/commit.json",
        "title": "NAP Commit",
        "description": concat!(
            "A NAP commit records a point-in-time snapshot of a manifest ",
            "plus patch metadata describing what changed."
        ),
        "type": "object",
        "required": ["id", "timestamp", "author", "message", "manifest_hash"],
        "properties": {
            "id": {
                "type": "string",
                "pattern": "^[a-f0-9]{64}$",
                "description": "BLAKE3 content-addressed commit identifier."
            },
            "parent": {
                "type": "string",
                "pattern": "^[a-f0-9]{64}$",
                "description": "Parent commit hash. null for the initial commit."
            },
            "timestamp": {
                "type": "string",
                "format": "date-time",
                "description": "ISO 8601 timestamp of when the commit was created."
            },
            "author": {
                "type": "string",
                "description": concat!(
                    "Author identifier ",
                    "(DID key, email, or key fingerprint)."
                )
            },
            "signature": {
                "type": "string",
                "description": concat!(
                    "Ed25519 signature over the commit hash ",
                    "(optional in v0)."
                )
            },
            "message": {
                "type": "string",
                "description": "Human-readable commit message describing the changes."
            },
            "manifest_hash": {
                "type": "string",
                "pattern": "^blake3:[a-f0-9]{64}$",
                "description": concat!(
                    "BLAKE3 content hash of the resulting manifest ",
                    "after this commit."
                )
            },
            "changes": {
                "type": "array",
                "items": {
                    "$ref": "#/definitions/Change"
                },
                "description": "Patch metadata describing what changed in this commit."
            }
        },
        "definitions": {
            "Change": {
                "type": "object",
                "required": ["path", "operation"],
                "properties": {
                    "path": {
                        "type": "string",
                        "description": concat!(
                            "Dot-notation path to the changed field. ",
                            "e.g., 'properties.homeworld', ",
                            "'representations.reference_image.hash'"
                        )
                    },
                    "operation": {
                        "type": "string",
                        "enum": ["set", "delete", "append", "remove"],
                        "description": "The kind of change operation."
                    },
                    "old_value": {
                        "type": "string",
                        "description": "Previous value hash (for verification)."
                    },
                    "new_value": {
                        "type": "string",
                        "description": "New value hash or literal."
                    }
                }
            }
        }
    })
}

/// Validate a manifest against the manifest schema.
///
/// Returns `Ok(())` if valid, or `Err` with a list of human-readable
/// validation error messages (path + description for each violation).
pub fn validate_manifest(manifest: &crate::manifest::Manifest) -> Result<(), Vec<String>> {
    let schema = manifest_schema();
    let instance = serde_json::to_value(manifest)
        .map_err(|e| vec![format!("manifest serialization error: {e}")])?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| vec![format!("schema compilation error: {e}")])?;
    let errors: Vec<String> = validator
        .iter_errors(&instance)
        .map(|e| format!("{}: {}", e.instance_path, e))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate a commit against the commit schema.
///
/// Returns `Ok(())` if valid, or `Err` with a list of human-readable
/// validation error messages.
pub fn validate_commit(commit: &crate::commit::Commit) -> Result<(), Vec<String>> {
    let schema = commit_schema();
    let instance = serde_json::to_value(commit)
        .map_err(|e| vec![format!("commit serialization error: {e}")])?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| vec![format!("schema compilation error: {e}")])?;
    let errors: Vec<String> = validator
        .iter_errors(&instance)
        .map(|e| format!("{}: {}", e.instance_path, e))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_manifest() {
        use crate::manifest::Manifest;
        let m = Manifest::new(
            "starwars",
            crate::types::EntityType::new("character"),
            "luke",
            "Luke Skywalker",
        );
        assert!(validate_manifest(&m).is_ok());
    }

    #[test]
    fn test_validate_manifest_with_custom_entity_type() {
        use crate::manifest::Manifest;
        let m = Manifest::new(
            "lab",
            crate::types::EntityType::new("scientific_paper"),
            "fusion-2024",
            "Cold Fusion Results",
        );
        assert!(validate_manifest(&m).is_ok());
    }

    #[test]
    fn test_validate_manifest_rejects_empty_entity_type() {
        // Build a manifest-like JSON value with an empty entity_type.
        let json = serde_json::json!({
            "id": "nap://starwars/character/luke",
            "name": "Luke",
            "entity_type": "",
            "version": 1,
        });
        // Validate the raw JSON value directly against the schema
        let schema = manifest_schema();
        let validator = jsonschema::validator_for(&schema).unwrap();
        let errors: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{}: {}", e.instance_path, e))
            .collect();
        assert!(
            !errors.is_empty(),
            "expected validation errors for empty entity_type"
        );
    }

    #[test]
    fn test_manifest_schema_is_valid_json() {
        let schema = manifest_schema();
        // Verify it's an object with required top-level keys
        assert!(schema.is_object());
        assert!(schema.get("title").is_some());
        assert!(schema.get("properties").is_some());
        assert!(schema.get("definitions").is_some());

        // Verify entity_type is a string type (no enum constraint)
        let props = schema.get("properties").unwrap();
        let et = props.get("entity_type").unwrap();
        assert_eq!(et.get("type").unwrap().as_str(), Some("string"));
        assert!(et.get("enum").is_none());
    }

    #[test]
    fn test_commit_schema_is_valid_json() {
        let schema = commit_schema();
        assert!(schema.is_object());
        assert!(schema.get("title").is_some());
        assert!(schema.get("properties").is_some());
        assert!(schema.get("definitions").is_some());
    }

    #[test]
    fn test_schemas_serialize_to_valid_json() {
        let m = manifest_schema();
        let json = serde_json::to_string(&m).unwrap();
        // Parse it back — if it fails, it's not valid JSON
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed.get("title").unwrap().as_str().unwrap(),
            "NAP Manifest"
        );

        let c = commit_schema();
        let json = serde_json::to_string(&c).unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.get("title").unwrap().as_str().unwrap(), "NAP Commit");
    }
}
