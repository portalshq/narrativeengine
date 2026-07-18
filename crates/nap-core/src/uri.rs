//! NAP URI parser and builder.
//!
//! The NAP URI scheme identifies narrative resources:
//!
//! ```text
//! nap://starwars/character/lukeskywalker#appearances.audienceVotes
//! ───┬── ───┬──── ────┬──── ──────┬────── ─────────────┬───────────
//!  scheme universe  entity_type entity_id          fragment (query)
//! ```
//!
//! **Key design decisions:**
//! - Version/branch/tag are NEVER encoded in the URI path. They are orthogonal
//!   selectors passed alongside the URI (mirrors Git, OCI, package managers).
//! - Fragment (`#`) carries the query path for subtree extraction.
//! - Entity type is any non-empty string — fully dynamic and user-defined.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::NapError;
use crate::types::EntityType;

/// The `nap://` URI scheme constant.
pub const NAP_SCHEME: &str = "nap://";

/// A parsed NAP URI representing a narrative resource identity.
///
/// # Examples
///
/// ```
/// use nap_core::uri::NapUri;
///
/// let uri: NapUri = "nap://starwars/character/lukeskywalker#references.appears_in"
///     .parse()
///     .unwrap();
///
/// assert_eq!(uri.universe, "starwars");
/// assert_eq!(uri.entity_type.as_str(), "character");
/// assert_eq!(uri.entity_id, "lukeskywalker");
/// assert_eq!(uri.fragment.as_deref(), Some("references.appears_in"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NapUri {
    /// The repository name (directory under base_dir). e.g., `"starwars"`, `"pokemon"`.
    pub universe: String,

    /// The kind of entity being addressed. Any non-empty string is valid.
    pub entity_type: EntityType,

    /// The entity's identifier (slug). e.g., `"lukeskywalker"`, `"pikachu"`.
    pub entity_id: String,

    /// Optional fragment for subtree queries. e.g., `"appearances.audienceVotes"`.
    /// Populated from the `#` portion of the URI.
    pub fragment: Option<String>,
}

impl NapUri {
    /// Construct a new NAP URI without a fragment.
    pub fn new(
        universe: impl Into<String>,
        entity_type: impl Into<EntityType>,
        entity_id: impl Into<String>,
    ) -> Self {
        Self {
            universe: universe.into(),
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
            fragment: None,
        }
    }

    /// Construct a NAP URI with a fragment query path.
    pub fn with_fragment(
        universe: impl Into<String>,
        entity_type: impl Into<EntityType>,
        entity_id: impl Into<String>,
        fragment: impl Into<String>,
    ) -> Self {
        Self {
            universe: universe.into(),
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
            fragment: Some(fragment.into()),
        }
    }

    /// Returns the canonical URI string WITHOUT the fragment.
    /// This is the resource identity — fragments are query concerns.
    pub fn identity(&self) -> String {
        format!(
            "nap://{}/{}/{}",
            self.universe, self.entity_type, self.entity_id
        )
    }

    /// Returns the relative filesystem path for this entity's manifest within
    /// a repository.
    ///
    /// e.g., `"character/lukeskywalker.yaml"` or `"repository.yaml"` for repo metadata.
    pub fn manifest_path(&self) -> String {
        if self.entity_type.as_str() == "world" {
            "repository.yaml".to_string()
        } else {
            format!(
                "{}/{}.yaml",
                self.entity_type.directory_name(),
                self.entity_id
            )
        }
    }
}

impl fmt::Display for NapUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "nap://{}/{}/{}",
            self.universe, self.entity_type, self.entity_id
        )?;
        if let Some(ref fragment) = self.fragment {
            write!(f, "#{fragment}")?;
        }
        Ok(())
    }
}

impl FromStr for NapUri {
    type Err = NapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.trim();

        // ── Strip scheme (optional) ──────────────────────────────────────
        // Accept both "nap://starwars/character/luke" and "starwars/character/luke".
        let without_scheme = input.strip_prefix(NAP_SCHEME).unwrap_or(input);

        // ── Split fragment ──────────────────────────────────────────────
        let (path_part, fragment) = match without_scheme.split_once('#') {
            Some((path, frag)) => {
                let frag_trimmed = frag.trim();
                if frag_trimmed.is_empty() {
                    (path, None)
                } else {
                    (path, Some(frag_trimmed.to_string()))
                }
            }
            None => (without_scheme, None),
        };

        // ── Parse path segments: universe / entity_type / entity_id ─────
        let segments: Vec<&str> = path_part.split('/').filter(|s| !s.is_empty()).collect();

        if segments.len() < 3 {
            return Err(NapError::InvalidUri {
                uri: input.to_string(),
                reason: format!(
                    "expected at least 3 path segments (universe/entity_type/entity_id), got {}",
                    segments.len()
                ),
            });
        }

        let universe = segments[0].to_string();
        let entity_type = EntityType::new(segments[1]);
        // Join remaining segments to support entity IDs with slashes (defensive)
        let entity_id = segments[2..].join("/");

        if universe.is_empty() {
            return Err(NapError::InvalidUri {
                uri: input.to_string(),
                reason: "universe name cannot be empty".to_string(),
            });
        }
        if entity_id.is_empty() {
            return Err(NapError::InvalidUri {
                uri: input.to_string(),
                reason: "entity ID cannot be empty".to_string(),
            });
        }

        Ok(NapUri {
            universe,
            entity_type,
            entity_id,
            fragment,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_uri_with_fragment() {
        let uri: NapUri = "nap://starwars/character/lukeskywalker#appearances.audienceVotes"
            .parse()
            .unwrap();
        assert_eq!(uri.universe, "starwars");
        assert_eq!(uri.entity_type.as_str(), "character");
        assert_eq!(uri.entity_id, "lukeskywalker");
        assert_eq!(uri.fragment.as_deref(), Some("appearances.audienceVotes"));
    }

    #[test]
    fn test_parse_uri_without_fragment() {
        let uri: NapUri = "nap://toystory/location/pizzapalace".parse().unwrap();
        assert_eq!(uri.universe, "toystory");
        assert_eq!(uri.entity_type.as_str(), "location");
        assert_eq!(uri.entity_id, "pizzapalace");
        assert!(uri.fragment.is_none());
    }

    #[test]
    fn test_parse_custom_entity_type() {
        let uri: NapUri = "nap://lab/paper/cold-fusion-v2".parse().unwrap();
        assert_eq!(uri.universe, "lab");
        assert_eq!(uri.entity_type.as_str(), "paper");
        assert_eq!(uri.entity_id, "cold-fusion-v2");
    }

    #[test]
    fn test_parse_scene_uri() {
        let uri: NapUri = "nap://starwars/scene/cantina".parse().unwrap();
        assert_eq!(uri.entity_type.as_str(), "scene");
        assert_eq!(uri.entity_id, "cantina");
    }

    #[test]
    fn test_parse_world_uri() {
        let uri: NapUri = "nap://starwars/world/starwars".parse().unwrap();
        assert_eq!(uri.entity_type.as_str(), "world");
    }

    #[test]
    fn test_roundtrip_display_parse() {
        let original = NapUri::with_fragment(
            "starwars",
            EntityType::new("character"),
            "lukeskywalker",
            "references.appears_in",
        );
        let displayed = original.to_string();
        let parsed: NapUri = displayed.parse().unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_identity_strips_fragment() {
        let uri = NapUri::with_fragment(
            "starwars",
            EntityType::new("character"),
            "lukeskywalker",
            "appearances",
        );
        assert_eq!(uri.identity(), "nap://starwars/character/lukeskywalker");
    }

    #[test]
    fn test_manifest_path_character() {
        let uri = NapUri::new("starwars", EntityType::new("character"), "lukeskywalker");
        assert_eq!(uri.manifest_path(), "character/lukeskywalker.yaml");
    }

    #[test]
    fn test_manifest_path_world() {
        let uri = NapUri::new("starwars", EntityType::new("world"), "starwars");
        assert_eq!(uri.manifest_path(), "repository.yaml");
    }

    #[test]
    fn test_manifest_path_custom_type() {
        let uri = NapUri::new("lab", EntityType::new("paper"), "cold-fusion-v2");
        assert_eq!(uri.manifest_path(), "paper/cold-fusion-v2.yaml");
    }

    #[test]
    fn test_invalid_too_few_segments() {
        let result = "nap://starwars/character".parse::<NapUri>();
        assert!(result.is_err());
    }

    #[test]
    fn test_optional_scheme() {
        let uri: NapUri = "starwars/character/lukeskywalker#references.appears_in"
            .parse()
            .unwrap();
        assert_eq!(uri.universe, "starwars");
        assert_eq!(uri.entity_type.as_str(), "character");
        assert_eq!(uri.entity_id, "lukeskywalker");
        assert_eq!(uri.fragment.as_deref(), Some("references.appears_in"));
    }

    #[test]
    fn test_bare_path_no_fragment() {
        let uri: NapUri = "toystory/location/pizzapalace".parse().unwrap();
        assert_eq!(uri.universe, "toystory");
        assert_eq!(uri.entity_type.as_str(), "location");
        assert_eq!(uri.entity_id, "pizzapalace");
        assert!(uri.fragment.is_none());
    }

    #[test]
    fn test_bare_path_too_few_segments() {
        let result = "starwars/character".parse::<NapUri>();
        assert!(result.is_err());
    }
}
