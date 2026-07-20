//! NAP Resolver — URI → Manifest, with query and version selectors.
//!
//! The resolver is the primary interface for reading NAP resources.
//! It handles:
//! - Full manifest resolution: `nap://starwars/character/lukeskywalker`
//! - Fragment queries: `nap://starwars/character/lukeskywalker#references.appears_in`
//! - Version selectors: branch, commit
//! - Subtree extraction for efficient AI/application access
//!
//! Version and branch are NEVER in the URI. They are orthogonal selectors:
//! ```text
//! URI + Reference + Revision Selector
//! ```

use std::path::{Path, PathBuf};

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

    /// Resolve at a specific tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

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
    /// Subtree result from a query.
    Subtree(serde_json::Value),
}

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
    /// ├── starwars/    ← repository repo
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
    /// resolver.resolve("nap://starwars/character/lukeskywalker", &Default::default())
    ///
    /// // Without scheme (auto-normalized)
    /// resolver.resolve("starwars/character/lukeskywalker", &Default::default())
    ///
    /// // With branch
    /// resolver.resolve("nap://starwars/character/lukeskywalker", &ResolveOptions {
    ///     branch: Some("canon".to_string()),
    ///     ..Default::default()
    /// })
    ///
    /// // With fragment query (via URI)
    /// resolver.resolve("nap://starwars/character/lukeskywalker#references.appears_in", &Default::default())
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

        // Handle recursive resolution
        if options.recursive.unwrap_or(false) {
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
    use crate::test_utils::MockBackend;
    use crate::types::EntityType;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Resolver) {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("starwars");
        let repo = Repository::init(&repo_path, "starwars", Box::new(MockBackend::new())).unwrap();

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
            serde_yaml::Value::String("nap://starwars/location/tatooine".to_string()),
        );
        manifest.add_reference(
            "appears_in",
            serde_yaml::Value::Sequence(vec![serde_yaml::Value::String(
                "nap://starwars/scene/cantina".to_string(),
            )]),
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
                "nap://starwars/character/lukeskywalker",
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

    #[test]
    fn test_resolve_with_fragment() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve(
                "nap://starwars/character/lukeskywalker#properties.species",
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
                "nap://starwars/character/lukeskywalker",
                &ResolveOptions {
                    path: Some("properties.homeworld".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        match result {
            ResolveResult::Subtree(v) => {
                assert_eq!(v.as_str(), Some("nap://starwars/location/tatooine"));
            }
            _ => panic!("expected subtree"),
        }
    }

    #[test]
    fn test_query_convenience() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .query(
                "nap://starwars/character/lukeskywalker",
                "properties.species",
            )
            .unwrap();
        assert_eq!(result.as_str(), Some("human"));
    }

    #[test]
    fn test_list_repositories() {
        let (_tmp, resolver) = setup();
        let repositories = resolver.list_repositories().unwrap();
        assert!(repositories.contains(&"starwars".to_string()));
    }

    #[test]
    fn test_resolve_not_found() {
        let (_tmp, resolver) = setup();
        let result = resolver.resolve("nap://starwars/character/nonexistent", &Default::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_without_scheme() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve("starwars/character/lukeskywalker", &Default::default())
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
                "starwars/character/lukeskywalker#properties.species",
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
            .resolve("starwars/character/lukeskywalker", &Default::default())
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
            .resolve("/starwars/character/lukeskywalker", &Default::default())
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
