//! NAP Manifest — the core primitive.
//!
//! The manifest is the durable representation of a narrative resource.
//! It is:
//! - **Human-editable** — YAML, readable by worldbuilders
//! - **Machine-editable** — structured, schema-validated
//! - **Agent-readable** — subtree-queryable for AI workflows
//! - **Mergeable** — YAML maps merge cleanly
//! - **Portable** — no runtime dependency, just a file
//! - **Signable** — hash the content, sign the hash
//! - **Versionable** — the manifest IS what gets committed
//!
//! # Design: Manifest is current state. History is external.
//!
//! The manifest stores `head` (a pointer to the latest commit hash).
//! The full commit history lives in the VCS, NOT inside the manifest.
//! This prevents manifests from growing unboundedly with every edit.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::content::ContentHash;
use crate::error::NapError;
use crate::types::EntityType;

/// A NAP manifest — the canonical representation of a narrative resource.
///
/// # Example (YAML)
/// ```yaml
/// id: "nap://starwars/character/lukeskywalker"
/// name: "Luke Skywalker"
/// entity_type: character
/// version: 17
/// properties:
///   homeworld: "nap://starwars/location/tatooine"
///   species: human
/// representations:
///   reference_image:
///     hash: "blake3:af1349b9..."
///     format: png
/// head: "a72c9f3b..."
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// The canonical NAP URI for this resource.
    /// e.g., `"nap://starwars/character/lukeskywalker"`
    pub id: String,

    /// Human-readable name.
    pub name: String,

    /// The kind of entity this manifest describes.
    pub entity_type: EntityType,

    /// Monotonic version counter. Incremented on each commit.
    #[serde(default)]
    pub version: u64,

    /// Access control. Owners have full control. Optional for v0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principals: Option<Principal>,

    /// Entity-specific key-value properties.
    /// Character: personality, species, homeworld, etc.
    /// Scene: setting, time_of_day, mood, etc.
    /// Location: geography, atmosphere, etc.
    #[serde(default)]
    pub properties: BTreeMap<String, serde_yaml::Value>,

    /// Content-addressed representations of this entity.
    /// e.g., reference_image, voice_model, mesh, splat, etc.
    #[serde(default)]
    pub representations: BTreeMap<String, Representation>,

    /// Cross-references to other NAP resources.
    /// e.g., appears_in, relationships, contains, etc.
    #[serde(default)]
    pub references: BTreeMap<String, serde_yaml::Value>,

    /// AI generation provenance — which model, prompt, seed, etc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,

    /// Pointer to the latest VCS commit hash (BLAKE3). History lives in the VCS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,

    /// Arbitrary extension metadata. Future-proof escape hatch.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, serde_yaml::Value>,
}

/// Access control principals for a manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    /// Full control — can modify, transfer, delete.
    #[serde(default)]
    pub owners: Vec<String>,

    /// Can modify content but not transfer ownership.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub maintainers: Vec<String>,

    /// Can publish/distribute but not modify source.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub publishers: Vec<String>,
}

/// A content-addressed representation of an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Representation {
    /// BLAKE3 content hash. e.g., `"blake3:af1349b9..."`.
    pub hash: String,

    /// File format. e.g., `"png"`, `"glb"`, `"onnx"`, `"spz"`.
    pub format: String,

    /// Optional storage URI. e.g., `"gs://assets/starwars/luke/ref.png"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    /// Optional quality tier: draft, production, distribution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
}

/// AI generation provenance metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// Which AI model generated this entity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Content-addressed hash of the prompt used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_hash: Option<String>,

    /// Generation seed for reproducibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,

    /// Additional generation parameters.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, String>,

    /// What this entity was derived from (parent entity URI).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived_from: Option<String>,

    /// When this entity was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

impl Manifest {
    /// Create a new manifest with minimal required fields.
    pub fn new(universe: &str, entity_type: EntityType, entity_id: &str, name: &str) -> Self {
        let id = match entity_type {
            EntityType::World => format!("nap://{universe}/world/{universe}"),
            _ => format!("nap://{universe}/{entity_type}/{entity_id}"),
        };

        Self {
            id,
            name: name.to_string(),
            entity_type,
            version: 0,
            principals: None,
            properties: BTreeMap::new(),
            representations: BTreeMap::new(),
            references: BTreeMap::new(),
            provenance: None,
            head: None,
            metadata: BTreeMap::new(),
        }
    }

    /// Serialize this manifest to YAML.
    pub fn to_yaml(&self) -> Result<String, NapError> {
        serde_yaml::to_string(self).map_err(|e| NapError::ManifestValidationError(e.to_string()))
    }

    /// Deserialize a manifest from a YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Self, NapError> {
        serde_yaml::from_str(yaml).map_err(|e| NapError::ManifestParseError {
            path: "<string>".to_string(),
            source: e,
        })
    }

    /// Read a manifest from a YAML file on disk.
    pub fn from_file(path: &Path) -> Result<Self, NapError> {
        let content = std::fs::read_to_string(path)?;
        serde_yaml::from_str(&content).map_err(|e| NapError::ManifestParseError {
            path: path.display().to_string(),
            source: e,
        })
    }

    /// Write this manifest to a YAML file on disk.
    pub fn to_file(&self, path: &Path) -> Result<(), NapError> {
        let yaml = self.to_yaml()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, yaml).map_err(|e| NapError::ManifestWriteError {
            path: path.display().to_string(),
            source: e,
        })
    }

    /// Convert the manifest to a serde_yaml::Value for query traversal.
    pub fn to_value(&self) -> Result<serde_yaml::Value, NapError> {
        serde_yaml::to_value(self).map_err(|e| NapError::ManifestValidationError(e.to_string()))
    }

    /// Convert the manifest to a serde_json::Value for JSON output.
    pub fn to_json_value(&self) -> Result<serde_json::Value, NapError> {
        serde_json::to_value(self).map_err(|e| NapError::ManifestValidationError(e.to_string()))
    }

    /// Compute the BLAKE3 hash of this manifest's YAML representation.
    pub fn content_hash(&self) -> Result<ContentHash, NapError> {
        let yaml = self.to_yaml()?;
        Ok(ContentHash::from_str_content(&yaml))
    }

    /// Add or update a representation.
    pub fn set_representation(&mut self, key: &str, repr: Representation) {
        self.representations.insert(key.to_string(), repr);
    }

    /// Add or update a property.
    pub fn set_property(&mut self, key: &str, value: serde_yaml::Value) {
        self.properties.insert(key.to_string(), value);
    }

    /// Add a cross-reference.
    pub fn add_reference(&mut self, key: &str, value: serde_yaml::Value) {
        self.references.insert(key.to_string(), value);
    }

    /// Increment the version counter.
    pub fn bump_version(&mut self) {
        self.version += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_new() {
        let manifest = Manifest::new(
            "starwars",
            EntityType::Character,
            "lukeskywalker",
            "Luke Skywalker",
        );
        assert_eq!(manifest.id, "nap://starwars/character/lukeskywalker");
        assert_eq!(manifest.name, "Luke Skywalker");
        assert_eq!(manifest.entity_type, EntityType::Character);
        assert_eq!(manifest.version, 0);
    }

    #[test]
    fn test_manifest_yaml_roundtrip() {
        let mut manifest = Manifest::new(
            "starwars",
            EntityType::Character,
            "lukeskywalker",
            "Luke Skywalker",
        );
        manifest.set_property("species", serde_yaml::Value::String("human".to_string()));
        manifest.set_representation(
            "reference_image",
            Representation {
                hash: "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                format: "png".to_string(),
                uri: Some("gs://assets/luke/ref.png".to_string()),
                tier: Some("production".to_string()),
            },
        );

        let yaml = manifest.to_yaml().unwrap();
        let parsed = Manifest::from_yaml(&yaml).unwrap();

        assert_eq!(parsed.id, manifest.id);
        assert_eq!(parsed.name, manifest.name);
        assert!(parsed.properties.contains_key("species"));
        assert!(parsed.representations.contains_key("reference_image"));
    }

    #[test]
    fn test_manifest_world_uri() {
        let manifest = Manifest::new(
            "starwars",
            EntityType::World,
            "starwars",
            "Star Wars Universe",
        );
        assert_eq!(manifest.id, "nap://starwars/world/starwars");
    }

    #[test]
    fn test_manifest_content_hash_deterministic() {
        let manifest = Manifest::new(
            "starwars",
            EntityType::Character,
            "lukeskywalker",
            "Luke Skywalker",
        );
        let hash_a = manifest.content_hash().unwrap();
        let hash_b = manifest.content_hash().unwrap();
        assert_eq!(hash_a, hash_b);
    }
}
