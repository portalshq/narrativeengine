//! NAP Resolver — URI → Manifest, with query and version selectors.
//!
//! The resolver is the primary interface for reading NAP resources.
//! It handles:
//! - Full manifest resolution: `nap://toystory/character/lukeskywalker`
//! - Fragment queries: `nap://toystory/character/lukeskywalker#references.appears_in`
//! - Version selectors: branch, commit
//! - Subtree extraction for efficient AI/application access
//!
//! Version and branch are NEVER in the URI. They are orthogonal selectors:
//! ```text
//! URI + Reference + Revision Selector
//! ```

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::error::NapError;
use crate::manifest::Manifest;
use crate::query::ManifestQuery;
use crate::repository::Repository;
use crate::uri::NapUri;
use crate::vcs::VcsBackend;
use crate::vcs_lore::LoreBackend;

/// Resolver configuration — set at construction time.
///
/// Controls how the resolver resolves URIs when no explicit branch or
/// commit is provided by the caller.
#[derive(Debug, Clone, Default)]
pub struct ResolveConfig {
    /// Branch to resolve when neither `branch` nor `commit` is specified
    /// in [`ResolveOptions`].  If `None`, resolves without a branch or
    /// commit — this will trigger a [`NapError::NoDefaultBranch`] error
    /// for any resolve call that omits both `branch` and `commit`.
    pub default_branch: Option<String>,
}

/// Options for resolving a NAP URI. All are optional — omitting all
/// causes the resolver to use its [`ResolveConfig::default_branch`] (if
/// configured) or fail with [`NapError::NoDefaultBranch`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolveOptions {
    /// Resolve at a specific branch. e.g., `"canon"`.
    /// Takes precedence over [`ResolveConfig::default_branch`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Resolve at a specific commit hash (BLAKE3). e.g.,
    /// `"af1349b9f5f9a1a6a0404deb36d020949b834f2a42e37e5f8d2e4ba2765f1a2f"`.
    /// Takes precedence over `branch` and [`ResolveConfig::default_branch`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,

    /// Subtree query path (overrides URI fragment). e.g., `"appearances.audienceVotes"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Recursively resolve nested URIs. When true, the resolver will follow
    /// all nap:// URIs found in the resolved manifest and resolve them as well.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recursive: Option<bool>,

    /// Maximum recursion depth for recursive resolution. Defaults to 10 to prevent
    /// infinite loops. Set to None for unlimited depth (not recommended).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,

    /// Include per-file provenance metadata for the manifest and direct representations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<bool>,

    /// Hydrate known readable provenance artifacts such as prompts and run records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_blobs: Option<bool>,
}

impl ResolveOptions {
    /// Returns the query path (from options or URI fragment).
    fn query_path(&self, uri: &NapUri) -> Option<String> {
        self.path.clone().or_else(|| uri.fragment.clone())
    }
}

/// The result of resolving a NAP URI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResolveResult {
    /// Full manifest (no query applied).
    Full(Box<Manifest>),
    /// Full manifest with Lore-backed per-file provenance envelope.
    Provenance(Box<ResolveEnvelope>),
    /// Subtree result from a query.
    Subtree(serde_json::Value),
}

/// Envelope returned when `ResolveOptions::provenance` is enabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveEnvelope {
    pub manifest: Box<Manifest>,
    pub provenance: ResolveProvenanceEnvelope,
}

/// Per-resolution provenance metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveProvenanceEnvelope {
    pub revision: String,
    pub files: Vec<ResolveProvenanceFile>,
}

/// Provenance for one file participating in an entity resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveProvenanceFile {
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    pub provenance: serde_json::Value,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub blobs: BTreeMap<String, HydratedProvenanceBlob>,
}

/// Hydrated readable provenance artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydratedProvenanceBlob {
    pub address: String,
    pub content: String,
    pub truncated: bool,
    pub original_bytes: usize,
    pub included_bytes: usize,
}

const MAX_CONDENSED_METADATA_VALUE_BYTES: usize = 256;
const MAX_HYDRATED_BLOB_BYTES: usize = 12_000;

/// The NAP resolver — resolves URIs to manifests or subtrees.
pub struct Resolver {
    /// Base directory containing repository repositories.
    base_path: PathBuf,
    /// VCS backend factory (creates backend per-repo).
    vcs_factory: fn() -> Box<dyn VcsBackend>,
    /// Resolution configuration (default branch, etc.).
    config: ResolveConfig,
}

impl Resolver {
    /// Create a resolver that looks for repository repos under `base_path`.
    ///
    /// WARNING: Uses [`LoreBackend::from_env()`] by default. For testing,
    /// use [`Resolver::with_vcs_factory()`] with a mock backend.
    ///
    /// Uses [`ResolveConfig::default()`] — meaning `default_branch` is
    /// `None` and any resolve that omits both `branch` and `commit` will
    /// fail with [`NapError::NoDefaultBranch`].
    ///
    /// # Example layout
    /// ```text
    /// base_path/
    /// ├── toystory/    ← repository repo
    /// ├── toystory/    ← repository repo
    /// └── marvel/      ← repository repo
    /// ```
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
            vcs_factory: || Box::new(LoreBackend::from_env()),
            config: ResolveConfig::default(),
        }
    }

    /// Create a resolver with a custom VCS backend factory and config.
    pub fn with_vcs_factory(
        base_path: &Path,
        factory: fn() -> Box<dyn VcsBackend>,
        config: ResolveConfig,
    ) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
            vcs_factory: factory,
            config,
        }
    }

    /// Open the repository for a given repository and read its resolve config.
    fn open_repo(&self, repository: &str) -> Result<(Repository, ResolveConfig), NapError> {
        let repo_path = self.base_path.join(repository);
        let repo = Repository::open(&repo_path, (self.vcs_factory)())?;
        let repo_config = repo.read_resolve_config();
        Ok((repo, repo_config))
    }

    /// Resolve a NAP URI string with options.
    ///
    /// # Examples
    /// ```text
    /// // Full manifest
    /// resolver.resolve("nap://toystory/character/lukeskywalker", &Default::default())
    ///
    /// // Without scheme (auto-normalized)
    /// resolver.resolve("toystory/character/lukeskywalker", &Default::default())
    ///
    /// // With branch
    /// resolver.resolve("nap://toystory/character/lukeskywalker", &ResolveOptions {
    ///     branch: Some("canon".to_string()),
    ///     ..Default::default()
    /// })
    ///
    /// // With fragment query (via URI)
    /// resolver.resolve("nap://toystory/character/lukeskywalker#references.appears_in", &Default::default())
    /// ```
    pub fn resolve(
        &self,
        uri_str: &str,
        options: &ResolveOptions,
    ) -> Result<ResolveResult, NapError> {
        // ── Normalization: Prepend nap:// if missing ─────────────────────
        let normalized_uri_str = if uri_str.starts_with("nap://") {
            uri_str.to_string()
        } else {
            format!("nap://{}", uri_str.trim_start_matches('/'))
        };

        debug!(
            original_uri = %uri_str,
            normalized_uri = %normalized_uri_str,
            "normalized NAP URI"
        );

        let uri: NapUri = normalized_uri_str.parse()?;
        self.resolve_uri(&uri, options)
    }

    /// Resolve a parsed NAP URI with options.
    pub fn resolve_uri(
        &self,
        uri: &NapUri,
        options: &ResolveOptions,
    ) -> Result<ResolveResult, NapError> {
        debug!(
            uri = %uri,
            options = ?options,
            "resolving NAP URI"
        );

        let wants_provenance =
            options.provenance.unwrap_or(false) || options.include_blobs.unwrap_or(false);

        // Handle recursive resolution. Provenance is intentionally scoped to the
        // requested manifest and its direct representations, not related entities.
        if options.recursive.unwrap_or(false) && !wants_provenance {
            return self.resolve_uri_recursive(
                uri,
                options,
                0,
                &mut std::collections::HashSet::new(),
            );
        }

        self.resolve_uri_single(uri, options)
    }

    /// Resolve a single URI without recursion.
    fn resolve_uri_single(
        &self,
        uri: &NapUri,
        options: &ResolveOptions,
    ) -> Result<ResolveResult, NapError> {
        let (repo, repo_config) = self.open_repo(&uri.repository)?;
        let query_path = options.query_path(uri);

        // ── 4-Rule Resolution ────────────────────────────────────────
        // Rule 1: commit provided → use directly (bypass branch logic)
        // Rule 2: branch provided, no commit → resolve branch head
        // Rule 3: both null → use default_branch from repo config (fallback to global)
        // Rule 4: both null and no default_branch → hard error
        // ──────────────────────────────────────────────────────────────

        let revision = match (options.commit.as_ref(), options.branch.as_ref()) {
            (Some(commit), _) => {
                debug!(%commit, "resolve: rule 1 — commit provided");
                commit.clone()
            }
            (None, Some(branch)) => {
                debug!(%branch, "resolve: rule 2 — branch provided");
                repo.resolve_branch_head(branch)?
            }
            (None, None) => match &repo_config.default_branch {
                Some(default_branch) => {
                    debug!(%default_branch, "resolve: rule 3 — using repo default_branch");
                    repo.resolve_branch_head(default_branch)?
                }
                None => match &self.config.default_branch {
                    Some(global_default_branch) => {
                        debug!(%global_default_branch, "resolve: rule 3 — using global default_branch");
                        repo.resolve_branch_head(global_default_branch)?
                    }
                    None => {
                        debug!("resolve: rule 4 — no branch, no commit, no default_branch");
                        return Err(NapError::NoDefaultBranch);
                    }
                },
            },
        };

        // Read the manifest at the resolved revision
        let manifest = repo.read_manifest_at_ref(&uri.entity_type, &uri.entity_id, &revision)?;

        let wants_provenance =
            options.provenance.unwrap_or(false) || options.include_blobs.unwrap_or(false);
        if wants_provenance {
            if let Some(path) = query_path {
                return Err(NapError::Other(format!(
                    "provenance envelopes are only supported for full manifest resolution, not subtree query '{path}'"
                )));
            }

            let envelope = self.build_provenance_envelope(
                &repo,
                uri,
                manifest,
                &revision,
                options.include_blobs.unwrap_or(false),
            )?;
            info!(uri = %uri, "resolved NAP URI with provenance");
            return Ok(ResolveResult::Provenance(Box::new(envelope)));
        }

        // Apply query if present
        match query_path {
            Some(ref path) => {
                debug!(query_path = %path, "applying subtree query");
                let yaml_value = manifest.to_value()?;
                let result = ManifestQuery::query(&yaml_value, path, &manifest.id)?;

                // Convert YAML value to JSON for consistent API output
                let json_str = serde_yaml::to_string(&result)
                    .map_err(|e| NapError::ManifestValidationError(e.to_string()))?;
                let json_value: serde_json::Value = serde_yaml::from_str(&json_str)
                    .map_err(|e| NapError::ManifestValidationError(e.to_string()))?;

                info!(
                    uri = %uri,
                    query_path = %path,
                    "resolved NAP URI with query"
                );
                Ok(ResolveResult::Subtree(json_value))
            }
            None => {
                info!(uri = %uri, "resolved NAP URI (full manifest)");
                Ok(ResolveResult::Full(Box::new(manifest)))
            }
        }
    }

    fn build_provenance_envelope(
        &self,
        repo: &Repository,
        uri: &NapUri,
        manifest: Manifest,
        revision: &str,
        include_blobs: bool,
    ) -> Result<ResolveEnvelope, NapError> {
        let manifest_path = uri.manifest_path();
        let mut files = vec![self.build_provenance_file(
            repo,
            revision,
            "manifest",
            None,
            Some(manifest_path.clone()),
            None,
            None,
            None,
            include_blobs,
        )?];

        for (name, representation) in &manifest.representations {
            let resolved_path = representation
                .uri
                .as_deref()
                .map(|representation_uri| {
                    Self::resolve_representation_path(&manifest_path, representation_uri)
                })
                .transpose()?
                .flatten();

            files.push(self.build_provenance_file(
                repo,
                revision,
                "representation",
                Some(name.clone()),
                resolved_path,
                representation.uri.clone(),
                Some(representation.hash.clone()),
                Some(representation.format.clone()),
                include_blobs,
            )?);
        }

        Ok(ResolveEnvelope {
            manifest: Box::new(manifest),
            provenance: ResolveProvenanceEnvelope {
                revision: revision.to_string(),
                files,
            },
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn build_provenance_file(
        &self,
        repo: &Repository,
        revision: &str,
        role: &str,
        name: Option<String>,
        path: Option<String>,
        uri: Option<String>,
        hash: Option<String>,
        format: Option<String>,
        include_blobs: bool,
    ) -> Result<ResolveProvenanceFile, NapError> {
        let metadata = match path.as_deref() {
            Some(path) => repo
                .vcs()
                .file_metadata_at_ref(&repo.root, path, revision)?,
            None => None,
        };

        let blobs = if include_blobs {
            match metadata.as_ref() {
                Some(metadata) => Self::hydrate_known_blobs(repo, metadata)?,
                None => BTreeMap::new(),
            }
        } else {
            BTreeMap::new()
        };

        let provenance = match metadata {
            Some(metadata) => {
                let condensed = Self::condense_metadata(metadata);
                if condensed.is_empty() {
                    serde_json::Value::String("none".to_string())
                } else {
                    serde_json::to_value(condensed).map_err(|e| {
                        NapError::Other(format!("failed to serialize provenance metadata: {e}"))
                    })?
                }
            }
            None => serde_json::Value::String("none".to_string()),
        };

        Ok(ResolveProvenanceFile {
            role: role.to_string(),
            name,
            path,
            uri,
            hash,
            format,
            provenance,
            blobs,
        })
    }

    fn condense_metadata(metadata: BTreeMap<String, String>) -> BTreeMap<String, String> {
        metadata
            .into_iter()
            .filter(|(_, value)| value.len() <= MAX_CONDENSED_METADATA_VALUE_BYTES)
            .collect()
    }

    fn hydrate_known_blobs(
        repo: &Repository,
        metadata: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, HydratedProvenanceBlob>, NapError> {
        let known_blob_keys = [
            ("prompt", "nap.provenance.prompt.address"),
            ("run", "nap.provenance.run.address"),
            ("parameters", "nap.provenance.parameters.address"),
        ];

        let mut blobs = BTreeMap::new();
        for (name, metadata_key) in known_blob_keys {
            let Some(address) = metadata.get(metadata_key) else {
                continue;
            };
            let content = repo.vcs().read_provenance_blob(&repo.root, address)?;
            blobs.insert(name.to_string(), Self::truncate_blob(address, &content));
        }
        Ok(blobs)
    }

    fn truncate_blob(address: &str, content: &str) -> HydratedProvenanceBlob {
        let original_bytes = content.len();
        let mut included_bytes = 0;
        let mut truncated_content = String::new();

        for ch in content.chars() {
            let next_len = included_bytes + ch.len_utf8();
            if next_len > MAX_HYDRATED_BLOB_BYTES {
                break;
            }
            truncated_content.push(ch);
            included_bytes = next_len;
        }

        HydratedProvenanceBlob {
            address: address.to_string(),
            content: truncated_content,
            truncated: included_bytes < original_bytes,
            original_bytes,
            included_bytes,
        }
    }

    fn resolve_representation_path(
        manifest_path: &str,
        representation_uri: &str,
    ) -> Result<Option<String>, NapError> {
        if representation_uri.contains("://") {
            return Ok(None);
        }

        let representation_path = Path::new(representation_uri);
        if representation_path.is_absolute() {
            return Err(NapError::InvalidQueryPath(format!(
                "representation URI must be relative for provenance lookup: {representation_uri}"
            )));
        }

        let mut clean = PathBuf::new();
        for component in representation_path.components() {
            match component {
                Component::Normal(part) => clean.push(part),
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(NapError::InvalidQueryPath(format!(
                        "unsafe representation URI for provenance lookup: {representation_uri}"
                    )));
                }
            }
        }

        let manifest_dir = Path::new(manifest_path).parent().unwrap_or(Path::new(""));
        Ok(Some(Self::path_to_lore_path(&manifest_dir.join(clean))))
    }

    fn path_to_lore_path(path: &Path) -> String {
        path.components()
            .filter_map(|component| match component {
                Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    /// Resolve a URI recursively, following nested nap:// URIs.
    fn resolve_uri_recursive(
        &self,
        uri: &NapUri,
        options: &ResolveOptions,
        depth: usize,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<ResolveResult, NapError> {
        // Check depth limit
        let max_depth = options.max_depth.unwrap_or(10);
        if depth >= max_depth {
            debug!(depth, max_depth, "reached maximum recursion depth");
            return self.resolve_uri_single(uri, options);
        }

        // Check for circular references
        let uri_str = uri.to_string();
        if visited.contains(&uri_str) {
            debug!(uri = %uri_str, "detected circular reference, stopping recursion");
            return self.resolve_uri_single(uri, options);
        }
        visited.insert(uri_str.clone());

        debug!(uri = %uri_str, depth, "recursively resolving URI");

        // Resolve the current URI
        let result = self.resolve_uri_single(uri, options)?;

        // Extract nested URIs from the result and resolve them
        match result {
            ResolveResult::Full(manifest) => {
                let nested_uris = self.extract_nested_uris(&manifest);
                if nested_uris.is_empty() {
                    debug!(uri = %uri_str, "no nested URIs found, returning manifest");
                    return Ok(ResolveResult::Full(manifest));
                }

                debug!(uri = %uri_str, count = nested_uris.len(), "found nested URIs, resolving recursively");

                // Resolve nested URIs and merge them into the result
                let mut resolved_manifest = (*manifest).clone();
                for nested_uri in nested_uris {
                    let nested_uri_parsed: NapUri = nested_uri.parse()?;

                    let nested_result = self
                        .resolve_uri_recursive(&nested_uri_parsed, options, depth + 1, visited)
                        .map_err(|e| {
                            NapError::Other(format!(
                                "failed to resolve nested URI '{}' while resolving '{}': {}",
                                nested_uri, uri_str, e
                            ))
                        })?;

                    if let ResolveResult::Full(nested_manifest) = nested_result {
                        // Merge nested manifest into parent (simple merge for now)
                        // In the future, this could be more sophisticated based on schema
                        for (key, value) in nested_manifest.properties {
                            resolved_manifest.properties.insert(key, value);
                        }
                    }
                }

                Ok(ResolveResult::Full(Box::new(resolved_manifest)))
            }
            ResolveResult::Subtree(value) => {
                // For subtree queries, we don't recurse (would be complex to merge)
                debug!("subtree query, skipping recursive resolution");
                Ok(ResolveResult::Subtree(value))
            }
            ResolveResult::Provenance(envelope) => Ok(ResolveResult::Provenance(envelope)),
        }
    }

    /// Extract all nap:// URIs from a manifest.
    fn extract_nested_uris(&self, manifest: &Manifest) -> Vec<String> {
        let mut uris = Vec::new();

        // Search in properties
        for value in manifest.properties.values() {
            self.extract_uris_from_yaml_value(value, &mut uris);
        }

        // Search in references
        for value in manifest.references.values() {
            self.extract_uris_from_yaml_value(value, &mut uris);
        }

        // Deduplicate URIs to avoid resolving the same URI multiple times
        uris.sort();
        uris.dedup();
        uris
    }

    /// Recursively extract nap:// URIs from YAML values.
    fn extract_uris_from_yaml_value(&self, value: &serde_yaml::Value, uris: &mut Vec<String>) {
        match value {
            serde_yaml::Value::String(s) if s.starts_with("nap://") => {
                uris.push(s.clone());
            }
            serde_yaml::Value::Sequence(seq) => {
                for item in seq {
                    self.extract_uris_from_yaml_value(item, uris);
                }
            }
            serde_yaml::Value::Mapping(map) => {
                for (_, v) in map {
                    self.extract_uris_from_yaml_value(v, uris);
                }
            }
            _ => {}
        }
    }

    /// Convenience: query a specific path on a URI.
    pub fn query(&self, uri_str: &str, path: &str) -> Result<serde_json::Value, NapError> {
        let options = ResolveOptions {
            path: Some(path.to_string()),
            ..Default::default()
        };
        match self.resolve(uri_str, &options)? {
            ResolveResult::Subtree(v) => Ok(v),
            ResolveResult::Full(m) => m.to_json_value(),
            ResolveResult::Provenance(envelope) => serde_json::to_value(envelope).map_err(|e| {
                NapError::Other(format!("failed to serialize provenance envelope: {e}"))
            }),
        }
    }

    /// List all repositories available.
    pub fn list_repositories(&self) -> Result<Vec<String>, NapError> {
        let mut repositories = Vec::new();
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            // Check for repository.yaml or repository.yaml to identify valid repositories
            if path.is_dir()
                && (path.join("repository.yaml").exists() || path.join("repository.yaml").exists())
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
            {
                repositories.push(name.to_string());
            }
        }
        repositories.sort();
        Ok(repositories)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::manifest::Representation;
    use crate::test_utils::MockBackend;
    use crate::types::EntityType;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Resolver) {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("toystory");
        let repo = Repository::init(&repo_path, "toystory", Box::new(MockBackend::new())).unwrap();

        // Create a character
        let (mut manifest, _) = repo
            .create_entity(
                &EntityType::new("character"),
                "lukeskywalker",
                "Luke Skywalker",
                "test",
            )
            .unwrap();

        // Add properties and commit
        manifest.set_property("species", serde_yaml::Value::String("human".to_string()));
        manifest.set_property(
            "homeworld",
            serde_yaml::Value::String("nap://toystory/location/tatooine".to_string()),
        );
        manifest.add_reference(
            "appears_in",
            serde_yaml::Value::Sequence(vec![serde_yaml::Value::String(
                "nap://toystory/scene/cantina".to_string(),
            )]),
        );
        manifest.set_representation(
            "face_image",
            Representation {
                hash: "blake3:9753abf79e5aef60bd95ab76c1e5a14d01239beb37ff9897b6af8e040eb2413a"
                    .to_string(),
                format: "png".to_string(),
                uri: Some("face_image.png".to_string()),
                tier: None,
            },
        );

        use crate::commit::Change;
        repo.commit_manifest(
            &mut manifest,
            "add Luke Skywalker details",
            "test",
            vec![Change::set("properties.species", None, "human".to_string())],
        )
        .unwrap();

        let resolver = Resolver::with_vcs_factory(
            tmp.path(),
            || Box::new(MockBackend::new()),
            ResolveConfig {
                default_branch: Some("main".to_string()),
            },
        );
        (tmp, resolver)
    }

    #[test]
    fn test_resolve_full_manifest() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve(
                "nap://toystory/character/lukeskywalker",
                &Default::default(),
            )
            .unwrap();
        match result {
            ResolveResult::Full(m) => {
                assert_eq!(m.name, "Luke Skywalker");
                assert_eq!(m.entity_type.as_str(), "character");
            }
            _ => panic!("expected full manifest"),
        }
    }

    fn write_mock_metadata(repo_path: &Path, metadata: BTreeMap<String, BTreeMap<String, String>>) {
        std::fs::write(
            repo_path.join(".mock_file_metadata.json"),
            serde_json::to_string(&metadata).unwrap(),
        )
        .unwrap();
    }

    fn write_mock_blobs(repo_path: &Path, blobs: BTreeMap<String, String>) {
        std::fs::write(
            repo_path.join(".mock_provenance_blobs.json"),
            serde_json::to_string(&blobs).unwrap(),
        )
        .unwrap();
    }

    fn resolve_with_provenance(resolver: &Resolver) -> ResolveEnvelope {
        let result = resolver
            .resolve(
                "nap://toystory/character/lukeskywalker",
                &ResolveOptions {
                    provenance: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        match result {
            ResolveResult::Provenance(envelope) => *envelope,
            _ => panic!("expected provenance envelope"),
        }
    }

    #[test]
    fn test_resolve_with_provenance_returns_manifest_and_direct_file_entries() {
        let (tmp, resolver) = setup();
        let repo_path = tmp.path().join("toystory");
        write_mock_metadata(
            &repo_path,
            BTreeMap::from([
                (
                    "character/lukeskywalker.yaml".to_string(),
                    BTreeMap::from([
                        ("nap.provenance.kind".to_string(), "edit".to_string()),
                        ("nap.provenance.model".to_string(), "gpt-5".to_string()),
                        (
                            "nap.provenance.long".to_string(),
                            "x".repeat(MAX_CONDENSED_METADATA_VALUE_BYTES + 1),
                        ),
                    ]),
                ),
                (
                    "character/face_image.png".to_string(),
                    BTreeMap::from([("nap.provenance.kind".to_string(), "generation".to_string())]),
                ),
            ]),
        );

        let envelope = resolve_with_provenance(&resolver);
        assert_eq!(envelope.manifest.name, "Luke Skywalker");
        assert_eq!(envelope.provenance.files.len(), 2);

        let manifest_file = &envelope.provenance.files[0];
        assert_eq!(manifest_file.role, "manifest");
        assert_eq!(
            manifest_file.path.as_deref(),
            Some("character/lukeskywalker.yaml")
        );
        assert_eq!(manifest_file.provenance["nap.provenance.kind"], "edit");
        assert!(
            manifest_file
                .provenance
                .get("nap.provenance.long")
                .is_none()
        );

        let representation_file = &envelope.provenance.files[1];
        assert_eq!(representation_file.role, "representation");
        assert_eq!(representation_file.name.as_deref(), Some("face_image"));
        assert_eq!(
            representation_file.path.as_deref(),
            Some("character/face_image.png")
        );
        assert_eq!(representation_file.uri.as_deref(), Some("face_image.png"));
        assert_eq!(representation_file.format.as_deref(), Some("png"));
    }

    #[test]
    fn test_resolve_with_provenance_records_path_and_revision_metadata_lookups() {
        let (tmp, resolver) = setup();
        let repo_path = tmp.path().join("toystory");
        let envelope = resolve_with_provenance(&resolver);

        let requests: Vec<BTreeMap<String, String>> = serde_json::from_str(
            &std::fs::read_to_string(repo_path.join(".mock_metadata_requests.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(
            requests[0].get("path").unwrap(),
            "character/lukeskywalker.yaml"
        );
        assert_eq!(
            requests[0].get("revision").unwrap(),
            &envelope.provenance.revision
        );
        assert_eq!(requests[1].get("path").unwrap(), "character/face_image.png");
        assert_eq!(
            requests[1].get("revision").unwrap(),
            &envelope.provenance.revision
        );
        assert!(!requests.iter().any(|request| {
            request
                .get("path")
                .is_some_and(|path| path.starts_with("blake3:"))
        }));
    }

    #[test]
    fn test_resolve_with_provenance_uses_none_for_missing_metadata() {
        let (_tmp, resolver) = setup();
        let envelope = resolve_with_provenance(&resolver);
        assert_eq!(envelope.provenance.files[0].provenance, "none");
        assert_eq!(envelope.provenance.files[1].provenance, "none");
    }

    #[test]
    fn test_resolve_with_include_blobs_hydrates_known_readable_artifacts() {
        let (tmp, resolver) = setup();
        let repo_path = tmp.path().join("toystory");
        write_mock_metadata(
            &repo_path,
            BTreeMap::from([(
                "character/lukeskywalker.yaml".to_string(),
                BTreeMap::from([
                    (
                        "nap.provenance.prompt.address".to_string(),
                        "lore:prompt:1".to_string(),
                    ),
                    (
                        "unrelated.artifact.address".to_string(),
                        "lore:binary:1".to_string(),
                    ),
                ]),
            )]),
        );
        write_mock_blobs(
            &repo_path,
            BTreeMap::from([("lore:prompt:1".to_string(), "Describe Luke".to_string())]),
        );

        let result = resolver
            .resolve(
                "nap://toystory/character/lukeskywalker",
                &ResolveOptions {
                    provenance: Some(true),
                    include_blobs: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        let ResolveResult::Provenance(envelope) = result else {
            panic!("expected provenance envelope");
        };

        let blobs = &envelope.provenance.files[0].blobs;
        assert_eq!(blobs.len(), 1);
        assert_eq!(blobs["prompt"].content, "Describe Luke");
        assert!(!blobs["prompt"].truncated);
    }

    #[test]
    fn test_include_blobs_implies_provenance_envelope() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve(
                "nap://toystory/character/lukeskywalker",
                &ResolveOptions {
                    include_blobs: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(matches!(result, ResolveResult::Provenance(_)));
    }

    #[test]
    fn test_resolve_with_include_blobs_truncates_readable_artifacts() {
        let (tmp, resolver) = setup();
        let repo_path = tmp.path().join("toystory");
        write_mock_metadata(
            &repo_path,
            BTreeMap::from([(
                "character/lukeskywalker.yaml".to_string(),
                BTreeMap::from([(
                    "nap.provenance.prompt.address".to_string(),
                    "lore:prompt:large".to_string(),
                )]),
            )]),
        );
        write_mock_blobs(
            &repo_path,
            BTreeMap::from([(
                "lore:prompt:large".to_string(),
                "x".repeat(MAX_HYDRATED_BLOB_BYTES + 10),
            )]),
        );

        let result = resolver
            .resolve(
                "nap://toystory/character/lukeskywalker",
                &ResolveOptions {
                    provenance: Some(true),
                    include_blobs: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        let ResolveResult::Provenance(envelope) = result else {
            panic!("expected provenance envelope");
        };
        let blob = &envelope.provenance.files[0].blobs["prompt"];
        assert!(blob.truncated);
        assert_eq!(blob.original_bytes, MAX_HYDRATED_BLOB_BYTES + 10);
        assert_eq!(blob.included_bytes, MAX_HYDRATED_BLOB_BYTES);
        assert_eq!(blob.content.len(), MAX_HYDRATED_BLOB_BYTES);
    }

    #[test]
    fn test_provenance_rejects_unsafe_representation_paths() {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("toystory");
        let repo = Repository::init(&repo_path, "toystory", Box::new(MockBackend::new())).unwrap();
        let (mut manifest, _) = repo
            .create_entity(&EntityType::new("character"), "leia", "Leia Organa", "test")
            .unwrap();
        manifest.set_representation(
            "unsafe",
            Representation {
                hash: "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                format: "png".to_string(),
                uri: Some("../secret.png".to_string()),
                tier: None,
            },
        );
        use crate::commit::Change;
        repo.commit_manifest(
            &mut manifest,
            "add unsafe representation",
            "test",
            vec![Change::set(
                "representations.unsafe",
                None,
                "unsafe".to_string(),
            )],
        )
        .unwrap();

        let resolver = Resolver::with_vcs_factory(
            tmp.path(),
            || Box::new(MockBackend::new()),
            ResolveConfig {
                default_branch: Some("main".to_string()),
            },
        );
        let err = resolver
            .resolve(
                "nap://toystory/character/leia",
                &ResolveOptions {
                    provenance: Some(true),
                    ..Default::default()
                },
            )
            .unwrap_err();
        assert!(err.to_string().contains("unsafe representation URI"));
    }

    #[test]
    fn test_resolve_with_fragment() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve(
                "nap://toystory/character/lukeskywalker#properties.species",
                &Default::default(),
            )
            .unwrap();
        match result {
            ResolveResult::Subtree(v) => {
                assert_eq!(v.as_str(), Some("human"));
            }
            _ => panic!("expected subtree"),
        }
    }

    #[test]
    fn test_resolve_with_options_path() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve(
                "nap://toystory/character/lukeskywalker",
                &ResolveOptions {
                    path: Some("properties.homeworld".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        match result {
            ResolveResult::Subtree(v) => {
                assert_eq!(v.as_str(), Some("nap://toystory/location/tatooine"));
            }
            _ => panic!("expected subtree"),
        }
    }

    #[test]
    fn test_query_convenience() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .query(
                "nap://toystory/character/lukeskywalker",
                "properties.species",
            )
            .unwrap();
        assert_eq!(result.as_str(), Some("human"));
    }

    #[test]
    fn test_list_repositories() {
        let (_tmp, resolver) = setup();
        let repositories = resolver.list_repositories().unwrap();
        assert!(repositories.contains(&"toystory".to_string()));
    }

    #[test]
    fn test_resolve_not_found() {
        let (_tmp, resolver) = setup();
        let result = resolver.resolve("nap://toystory/character/nonexistent", &Default::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_without_scheme() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve("toystory/character/lukeskywalker", &Default::default())
            .unwrap();
        match result {
            ResolveResult::Full(m) => {
                assert_eq!(m.name, "Luke Skywalker");
                assert_eq!(m.entity_type.as_str(), "character");
            }
            _ => panic!("expected full manifest"),
        }
    }

    #[test]
    fn test_resolve_without_scheme_with_fragment() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve(
                "toystory/character/lukeskywalker#properties.species",
                &Default::default(),
            )
            .unwrap();
        match result {
            ResolveResult::Subtree(v) => {
                assert_eq!(v.as_str(), Some("human"));
            }
            _ => panic!("expected subtree"),
        }
    }

    #[test]
    fn test_resolve_without_leading_slash() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve("toystory/character/lukeskywalker", &Default::default())
            .unwrap();
        match result {
            ResolveResult::Full(m) => {
                assert_eq!(m.name, "Luke Skywalker");
            }
            _ => panic!("expected full manifest"),
        }
    }

    #[test]
    fn test_resolve_with_leading_slash_without_scheme() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve("/toystory/character/lukeskywalker", &Default::default())
            .unwrap();
        match result {
            ResolveResult::Full(m) => {
                assert_eq!(m.name, "Luke Skywalker");
            }
            _ => panic!("expected full manifest"),
        }
    }
}

#[cfg(all(test, feature = "lore-integration"))]
mod lore_tests {
    use super::*;
    use crate::types::EntityType;
    use crate::vcs_lore::LoreBackend;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tempfile::TempDir;

    fn unique_suffix() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    fn setup_lore() -> (TempDir, Resolver, String) {
        let repository = format!("lr-{}", unique_suffix());
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join(&repository);
        let repo =
            Repository::init(&repo_path, &repository, Box::new(LoreBackend::from_env())).unwrap();

        // Create a character
        let (mut manifest, _) = repo
            .create_entity(
                &EntityType::new("character"),
                "lukeskywalker",
                "Luke Skywalker",
                "test",
            )
            .unwrap();

        // Add properties and commit
        manifest.set_property("species", serde_yaml::Value::String("human".to_string()));
        use crate::commit::Change;
        repo.commit_manifest(
            &mut manifest,
            "add Luke Skywalker details",
            "test",
            vec![Change::set("properties.species", None, "human".to_string())],
        )
        .unwrap();

        let resolver = Resolver::with_vcs_factory(
            tmp.path(),
            || Box::new(LoreBackend::from_env()),
            ResolveConfig {
                default_branch: Some("main".to_string()),
            },
        );
        (tmp, resolver, repository)
    }

    #[test]
    fn test_resolve_lore_full_manifest() {
        let (_tmp, resolver, repository) = setup_lore();
        let uri = format!("nap://{}/character/lukeskywalker", repository);
        let result = resolver.resolve(&uri, &Default::default()).unwrap();
        match result {
            ResolveResult::Full(m) => {
                assert_eq!(m.name, "Luke Skywalker");
            }
            _ => panic!("expected full manifest"),
        }
    }
}
