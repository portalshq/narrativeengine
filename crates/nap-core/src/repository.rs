//! Universe repository — filesystem layout and manifest CRUD.
//!
//! A NAP repository represents a single fictional universe.
//! Repository structure:
//!
//! ```text
//! starwars/               ← universe root
//! ├── universe.yaml       ← world manifest (root-level, includes nap config in metadata)
//! ├── characters/
//! │   ├── lukeskywalker.yaml
//! │   └── darthvader.yaml
//! ├── locations/
//! │   └── tatooine.yaml
//! ├── scenes/
//! │   └── cantina-scene.yaml
//! └── props/
//! ```

use std::path::{Path, PathBuf};

use tracing::{debug, info};

use crate::commit::{Change, Commit};
use crate::error::NapError;
use crate::manifest::Manifest;
use crate::resolver::ResolveConfig;
use crate::types::EntityType;
use crate::uri::NapUri;
use crate::vcs::VcsBackend;

/// A NAP universe repository.
pub struct Repository {
    /// Filesystem path to the repository root.
    pub root: PathBuf,
    /// The universe name (derived from directory name).
    pub universe: String,
    /// The VCS backend (Lore).
    vcs: Box<dyn VcsBackend>,
}

impl Repository {
    /// Open an existing NAP repository at the given path.
    pub fn open(path: &Path, vcs: Box<dyn VcsBackend>) -> Result<Self, NapError> {
        // Check for universe.yaml to identify valid repository
        if !path.join("universe.yaml").exists() {
            return Err(NapError::RepositoryNotFound(path.display().to_string()));
        }

        let universe = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        debug!(
            path = %path.display(),
            universe = %universe,
            "opened NAP repository"
        );

        Ok(Self {
            root: path.to_path_buf(),
            universe,
            vcs,
        })
    }

    /// Read the resolve configuration from universe.yaml metadata.
    pub fn read_resolve_config(&self) -> ResolveConfig {
        let universe_path = self.root.join("universe.yaml");

        if !universe_path.exists() {
            debug!(
                path = %universe_path.display(),
                "universe.yaml not found, using default ResolveConfig"
            );
            return ResolveConfig::default();
        }

        let yaml_content = match std::fs::read_to_string(&universe_path) {
            Ok(content) => content,
            Err(e) => {
                debug!(
                    path = %universe_path.display(),
                    error = %e,
                    "failed to read universe.yaml, using default ResolveConfig"
                );
                return ResolveConfig::default();
            }
        };

        // Parse YAML and extract nap metadata
        let parsed: serde_yaml::Value = match serde_yaml::from_str(&yaml_content) {
            Ok(value) => value,
            Err(e) => {
                debug!(
                    path = %universe_path.display(),
                    error = %e,
                    "failed to parse universe.yaml, using default ResolveConfig"
                );
                return ResolveConfig::default();
            }
        };

        // Extract default_branch from nap metadata
        let default_branch = parsed
            .get("metadata")
            .and_then(|metadata| metadata.get("nap"))
            .and_then(|nap| nap.get("default_branch"))
            .and_then(|branch| branch.as_str())
            .map(|s| s.to_string());

        // Auto-create nap metadata if missing
        if default_branch.is_none() {
            debug!(
                path = %universe_path.display(),
                "nap metadata not found, auto-creating with default_branch = 'main'"
            );

            // Read the existing manifest
            let mut manifest = match Manifest::from_file(&universe_path) {
                Ok(m) => m,
                Err(e) => {
                    debug!(
                        path = %universe_path.display(),
                        error = %e,
                        "failed to read manifest, using default ResolveConfig"
                    );
                    return ResolveConfig::default();
                }
            };

            // Add nap metadata
            manifest.metadata.insert(
                "nap".to_string(),
                serde_yaml::to_value(serde_json::json!({
                    "default_branch": "main"
                }))
                .unwrap(),
            );

            // Write back to file
            if let Err(e) = manifest.to_file(&universe_path) {
                debug!(
                    path = %universe_path.display(),
                    error = %e,
                    "failed to write nap metadata, using default ResolveConfig"
                );
                return ResolveConfig::default();
            }

            return ResolveConfig {
                default_branch: Some("main".to_string()),
            };
        }

        ResolveConfig { default_branch }
    }

    /// Initialize a new NAP repository.
    pub fn init(path: &Path, universe: &str, vcs: Box<dyn VcsBackend>) -> Result<Self, NapError> {
        let repo_root = path.to_path_buf();
        if repo_root.join("universe.yaml").exists() {
            return Err(NapError::RepositoryAlreadyExists(
                repo_root.display().to_string(),
            ));
        }

        info!(
            path = %repo_root.display(),
            universe = %universe,
            "initializing NAP repository"
        );

        // Create directory structure
        std::fs::create_dir_all(&repo_root)?;

        // Create entity type subdirectories
        for entity_type in EntityType::subdirectory_types() {
            std::fs::create_dir_all(repo_root.join(entity_type.directory_name()))?;
        }

        // Create universe.yaml (world manifest) with [nap] metadata
        let mut world_manifest = Manifest::new(
            universe,
            EntityType::World,
            universe,
            &format!("{universe} Universe"),
        );

        // Add nap configuration to metadata
        world_manifest.metadata.insert(
            "nap".to_string(),
            serde_yaml::to_value(serde_json::json!({
                "default_branch": "main"
            }))
            .unwrap(),
        );

        world_manifest.to_file(&repo_root.join("universe.yaml"))?;

        // Initialize VCS
        vcs.init(&repo_root)?;

        // Initial commit
        vcs.commit(
            &repo_root,
            &format!("Initialize {universe} universe"),
            "nap-init",
        )?;

        info!(
            path = %repo_root.display(),
            universe = %universe,
            "NAP repository initialized successfully"
        );

        Ok(Self {
            root: repo_root,
            universe: universe.to_string(),
            vcs,
        })
    }

    /// Get the full filesystem path to an entity's manifest file.
    pub fn manifest_path(&self, entity_type: EntityType, entity_id: &str) -> PathBuf {
        let uri = NapUri::new(&self.universe, entity_type, entity_id);
        self.root.join(uri.manifest_path())
    }

    /// Read a manifest from the repository.
    pub fn read_manifest(
        &self,
        entity_type: EntityType,
        entity_id: &str,
    ) -> Result<Manifest, NapError> {
        let path = self.manifest_path(entity_type, entity_id);
        debug!(
            path = %path.display(),
            entity_type = %entity_type,
            entity_id = %entity_id,
            "reading manifest"
        );
        Manifest::from_file(&path)
    }

    /// Read a manifest at a specific VCS ref (commit, branch, tag).
    pub fn read_manifest_at_ref(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        reference: &str,
    ) -> Result<Manifest, NapError> {
        let uri = NapUri::new(&self.universe, entity_type, entity_id);
        let file_path = uri.manifest_path();

        debug!(
            file_path = %file_path,
            reference = %reference,
            "reading manifest at ref"
        );

        let content = self
            .vcs
            .read_file_at_ref(&self.root, &file_path, Some(reference))?;
        Manifest::from_yaml(&content)
    }

    /// Write a manifest to the repository (does NOT commit).
    pub fn write_manifest(&self, manifest: &Manifest) -> Result<PathBuf, NapError> {
        let uri: NapUri = manifest.id.parse()?;
        let path = self.root.join(uri.manifest_path());

        debug!(
            path = %path.display(),
            manifest_id = %manifest.id,
            "writing manifest"
        );

        manifest.to_file(&path)?;
        Ok(path)
    }

    /// Create a new entity manifest and commit it.
    pub fn create_entity(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        name: &str,
        author: &str,
    ) -> Result<(Manifest, String), NapError> {
        let mut manifest = Manifest::new(&self.universe, entity_type, entity_id, name);

        // Validate against schema before writing
        crate::schema::validate_manifest(&manifest)
            .map_err(|errors| NapError::ManifestValidationError(errors.join("; ")))?;

        // Write the manifest
        self.write_manifest(&manifest)?;

        // Commit via VCS
        let commit_message = format!("Create {entity_type} '{name}' ({entity_id})");
        let commit_hash = self.vcs.commit(&self.root, &commit_message, author)?;

        // Update manifest with head pointer
        manifest.head = Some(commit_hash.clone());
        manifest.bump_version();
        self.write_manifest(&manifest)?;

        info!(
            manifest_id = %manifest.id,
            commit_hash = %commit_hash,
            "created entity"
        );

        Ok((manifest, commit_hash))
    }

    /// Update an existing manifest and commit the changes.
    pub fn commit_manifest(
        &self,
        manifest: &mut Manifest,
        message: &str,
        author: &str,
        changes: Vec<Change>,
    ) -> Result<Commit, NapError> {
        // Validate against schema before writing
        crate::schema::validate_manifest(manifest)
            .map_err(|errors| NapError::ManifestValidationError(errors.join("; ")))?;

        // Bump version
        manifest.bump_version();

        // Write updated manifest (without new head — we don't know it yet)
        self.write_manifest(manifest)?;

        // Compute manifest hash
        let manifest_hash = manifest.content_hash()?.as_str().to_string();

        // VCS commit — produces the new HEAD hash
        let vcs_hash = self.vcs.commit(&self.root, message, author)?;

        // Create NAP commit object with the now-known VCS hash
        let nap_commit = Commit::new(
            manifest.head.clone(),
            author,
            message,
            &manifest_hash,
            changes,
        );

        // Update head pointer and write again (leaves working tree dirty,
        // same pattern as create_entity)
        manifest.head = Some(vcs_hash.clone());
        self.write_manifest(manifest)?;

        debug!(
            manifest_id = %manifest.id,
            version = manifest.version,
            nap_commit_id = %nap_commit.id,
            vcs_hash = %vcs_hash,
            "manifest committed"
        );

        Ok(nap_commit)
    }

    /// Get the commit history for a specific entity.
    pub fn history(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        limit: usize,
    ) -> Result<Vec<crate::vcs::CommitInfo>, NapError> {
        let uri = NapUri::new(&self.universe, entity_type, entity_id);
        let file_path = uri.manifest_path();
        self.vcs.log(&self.root, Some(&file_path), limit)
    }

    /// List all entity IDs of a given type in the repository.
    pub fn list_entities(&self, entity_type: EntityType) -> Result<Vec<String>, NapError> {
        let dir = self.root.join(entity_type.directory_name());
        if !dir.exists() {
            return Ok(vec![]);
        }

        let mut entities = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yaml")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                entities.push(stem.to_string());
            }
        }
        entities.sort();
        Ok(entities)
    }

    /// Delete an entity manifest and commit the deletion.
    pub fn delete_entity(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        author: &str,
    ) -> Result<String, NapError> {
        let path = self.manifest_path(entity_type, entity_id);
        if !path.exists() {
            return Err(NapError::ManifestNotFound(path.display().to_string()));
        }

        std::fs::remove_file(&path)?;

        let message = format!("Delete {entity_type} '{entity_id}'");
        let hash = self.vcs.commit(&self.root, &message, author)?;
        info!(entity_type = %entity_type, entity_id = %entity_id, "deleted entity");
        Ok(hash)
    }

    /// Create a branch in the underlying VCS.
    pub fn create_branch(&self, name: &str) -> Result<(), NapError> {
        self.vcs.create_branch(&self.root, name)
    }

    /// Switch to a branch.
    pub fn switch_branch(&self, name: &str) -> Result<(), NapError> {
        self.vcs.switch_branch(&self.root, name)
    }

    /// Create a tag.
    pub fn create_tag(&self, name: &str) -> Result<(), NapError> {
        self.vcs.create_tag(&self.root, name)
    }

    /// List branches.
    pub fn list_branches(&self) -> Result<Vec<String>, NapError> {
        self.vcs.list_branches(&self.root)
    }

    /// List tags.
    pub fn list_tags(&self) -> Result<Vec<String>, NapError> {
        self.vcs.list_tags(&self.root)
    }

    /// Revert a commit by creating a new VCS commit that undoes the specified one.
    ///
    /// The revert is a universe-level operation (not entity-scoped).
    /// After reverting, working-tree files are restored to their pre-commit content
    /// and a new revert commit is created in VCS history.
    pub fn revert_commit(&self, commit_hash: &str, author: &str) -> Result<String, NapError> {
        let new_hash = self.vcs.revert(&self.root, commit_hash)?;

        // Re-read all entity manifests and update their `head` pointer
        // so manifests are consistent with the new VCS state.
        for et in EntityType::subdirectory_types() {
            if let Ok(ids) = self.list_entities(*et) {
                for id in &ids {
                    if let Ok(mut manifest) = self.read_manifest(*et, id) {
                        manifest.head = Some(new_hash.clone());
                        self.write_manifest(&manifest).ok();
                    }
                }
            }
        }

        info!(
            commit = %commit_hash,
            revert = %new_hash,
            author = %author,
            "commit reverted"
        );

        Ok(new_hash)
    }

    /// Get current HEAD hash.
    pub fn head_hash(&self) -> Result<String, NapError> {
        self.vcs.head_hash(&self.root)
    }

    /// Resolve the most recent commit hash on a given branch.
    pub fn resolve_branch_head(&self, branch: &str) -> Result<String, NapError> {
        self.vcs.resolve_branch_head(&self.root, branch)
    }

    // ── Remote operations ─────────────────────────────────────────

    /// Add a remote to the repository.
    pub fn add_remote(&self, name: &str, url: &str) -> Result<(), NapError> {
        self.vcs.add_remote(&self.root, name, url)
    }

    /// Remove a remote from the repository.
    pub fn remove_remote(&self, name: &str) -> Result<(), NapError> {
        self.vcs.remove_remote(&self.root, name)
    }

    /// List remotes as `(name, url)` pairs.
    pub fn list_remotes(&self) -> Result<Vec<(String, String)>, NapError> {
        self.vcs.list_remotes(&self.root)
    }

    /// Push the current branch to a remote.
    pub fn push(&self, remote: Option<&str>, branch: Option<&str>) -> Result<(), NapError> {
        self.vcs.push(&self.root, remote, branch)
    }

    /// Pull the current branch from a remote.
    pub fn pull(&self, remote: Option<&str>, branch: Option<&str>) -> Result<(), NapError> {
        self.vcs.pull(&self.root, remote, branch)
    }

    /// Access the VCS backend (for the resolver to read files at specific refs).
    pub fn vcs(&self) -> &dyn VcsBackend {
        self.vcs.as_ref()
    }
}

// ── In-memory mock VcsBackend for testing ──────────────────────────────
// (Moved to nap-test-utils)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{MockBackend, mock_repo};
    use tempfile::TempDir;

    #[test]
    fn test_mock_backend_contract() {
        crate::test_utils::contract::run_repository_contract(MockBackend::new());
    }

    #[test]
    fn test_init_creates_structure() {
        let tmp = TempDir::new().unwrap();
        let repo = mock_repo(&tmp);

        assert!(repo.root.join("universe.yaml").exists());
        assert!(repo.root.join("characters").exists());
        assert!(repo.root.join("locations").exists());
        assert!(repo.root.join("scenes").exists());
        assert!(repo.root.join("props").exists());
    }

    #[test]
    fn test_create_and_read_entity() {
        let tmp = TempDir::new().unwrap();
        let repo = mock_repo(&tmp);

        let (manifest, _hash) = repo
            .create_entity(EntityType::Character, "hero", "The Hero", "test-author")
            .unwrap();

        assert_eq!(manifest.name, "The Hero");
        assert_eq!(manifest.entity_type, EntityType::Character);

        // Read it back
        let read_back = repo.read_manifest(EntityType::Character, "hero").unwrap();
        assert_eq!(read_back.name, "The Hero");
    }

    #[test]
    fn test_list_entities() {
        let tmp = TempDir::new().unwrap();
        let repo = mock_repo(&tmp);

        repo.create_entity(EntityType::Character, "alice", "Alice", "author")
            .unwrap();
        repo.create_entity(EntityType::Character, "bob", "Bob", "author")
            .unwrap();

        let chars = repo.list_entities(EntityType::Character).unwrap();
        assert_eq!(chars, vec!["alice", "bob"]);
    }

    #[test]
    fn test_commit_manifest_updates() {
        let tmp = TempDir::new().unwrap();
        let repo = mock_repo(&tmp);

        let (mut manifest, _) = repo
            .create_entity(EntityType::Character, "hero", "The Hero", "author")
            .unwrap();

        // Modify and commit
        manifest.set_property("species", serde_yaml::Value::String("elf".to_string()));
        let changes = vec![Change::set("properties.species", None, "elf".to_string())];
        let commit = repo
            .commit_manifest(&mut manifest, "set species to elf", "author", changes)
            .unwrap();

        assert!(!commit.id.is_empty());
        assert_eq!(commit.message, "set species to elf");

        // Verify version incremented
        let read_back = repo.read_manifest(EntityType::Character, "hero").unwrap();
        assert!(read_back.version >= 2);
    }

    #[test]
    fn test_history() {
        let tmp = TempDir::new().unwrap();
        let repo = mock_repo(&tmp);

        let (mut manifest, _) = repo
            .create_entity(EntityType::Character, "hero", "The Hero", "author")
            .unwrap();

        manifest.set_property(
            "name",
            serde_yaml::Value::String("Updated Hero".to_string()),
        );
        repo.commit_manifest(&mut manifest, "update name", "author", vec![])
            .unwrap();

        let hist = repo.history(EntityType::Character, "hero", 10).unwrap();
        assert!(hist.len() >= 2);
    }

    #[test]
    fn test_revert_commit() {
        let tmp = TempDir::new().unwrap();
        let repo = mock_repo(&tmp);

        // Create entity and note its name
        let (mut manifest, _) = repo
            .create_entity(EntityType::Character, "hero", "The Hero", "author")
            .unwrap();
        assert_eq!(manifest.name, "The Hero");

        // Modify and commit
        manifest.set_property("species", serde_yaml::Value::String("elf".to_string()));
        let changes = vec![Change::set("properties.species", None, "elf".to_string())];
        let _commit = repo
            .commit_manifest(&mut manifest, "set species to elf", "author", changes)
            .unwrap();

        // Verify the property was set
        let read_back = repo.read_manifest(EntityType::Character, "hero").unwrap();
        assert_eq!(
            read_back.properties.get("species").and_then(|v| v.as_str()),
            Some("elf")
        );

        // Get the VCS commit hash from the manifest's head pointer.
        // After commit_manifest, this is the single VCS commit containing
        // the property change (the head pointer update is left dirty).
        let vcs_hash = read_back
            .head
            .as_ref()
            .expect("head should be set after commit");

        // Revert that VCS commit
        let revert_hash = repo.revert_commit(vcs_hash, "author").unwrap();
        assert!(!revert_hash.is_empty());

        // Verify the manifest head was updated to the revert commit
        let after_revert = repo.read_manifest(EntityType::Character, "hero").unwrap();
        assert_eq!(after_revert.head.as_deref(), Some(revert_hash.as_str()));

        // Verify the revert appears in history
        let hist = repo.history(EntityType::Character, "hero", 10).unwrap();
        assert!(hist.iter().any(|c| c.id == revert_hash));
    }

    // fn test_remote_operations() {
    //     let tmp = TempDir::new().unwrap();
    //     let repo = mock_repo(&tmp);

    //     repo.add_remote("origin", "git@github.com:user/repo.git")
    //         .unwrap();
    //     let remotes = repo.list_remotes().unwrap();
    //     assert_eq!(remotes.len(), 1);
    //     assert_eq!(remotes[0].0, "origin");

    //     repo.remove_remote("origin").unwrap();
    //     let remotes = repo.list_remotes().unwrap();
    //     assert!(remotes.is_empty());
    // }
}

// ── Integration tests: Repository + LoreBackend ─────────────────────
// These require a running Lore server. Run with:
//   cargo test --features lore-integration
#[cfg(all(test, feature = "lore-integration"))]
mod lore_integration_tests {
    use super::*;
    use crate::vcs_lore::LoreBackend;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tempfile::TempDir;

    fn unique_suffix() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    fn setup_lore_repo() -> (TempDir, Repository) {
        let universe = format!("ri-{}", unique_suffix());
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join(&universe);
        let repo =
            Repository::init(&repo_path, &universe, Box::new(LoreBackend::from_env())).unwrap();
        (tmp, repo)
    }

    #[test]
    fn test_lore_init_creates_structure() {
        let (_tmp, repo) = setup_lore_repo();
        assert!(repo.root.join(".nap").exists());
        assert!(repo.root.join("universe.yaml").exists());
    }

    #[test]
    fn test_lore_create_and_read_entity() {
        let (_tmp, repo) = setup_lore_repo();

        let (manifest, _hash) = repo
            .create_entity(
                EntityType::Character,
                "hero",
                "Test Hero",
                "integration-test",
            )
            .unwrap();
        assert_eq!(manifest.name, "Test Hero");

        let read_back = repo.read_manifest(EntityType::Character, "hero").unwrap();
        assert_eq!(read_back.name, "Test Hero");
    }

    #[test]
    fn test_lore_commit_and_branch() {
        let (_tmp, repo) = setup_lore_repo();

        let (mut manifest, _) = repo
            .create_entity(
                EntityType::Character,
                "hero",
                "Test Hero",
                "integration-test",
            )
            .unwrap();

        manifest.set_property("species", serde_yaml::Value::String("human".to_string()));
        let changes = vec![Change::set("properties.species", None, "human".to_string())];
        repo.commit_manifest(&mut manifest, "add species", "integration-test", changes)
            .unwrap();

        let read_back = repo.read_manifest(EntityType::Character, "hero").unwrap();
        assert_eq!(read_back.version, 2);

        repo.create_branch("feature-branch").unwrap();
        let branches = repo.list_branches().unwrap();
        assert!(branches.contains(&"feature-branch".to_string()));
    }

    #[test]
    fn test_lore_delete_entity() {
        let (_tmp, repo) = setup_lore_repo();

        repo.create_entity(
            EntityType::Character,
            "hero",
            "Test Hero",
            "integration-test",
        )
        .unwrap();

        repo.delete_entity(EntityType::Character, "hero", "integration-test")
            .unwrap();

        let entities = repo.list_entities(EntityType::Character).unwrap();
        assert!(!entities.contains(&"hero".to_string()));
    }

    #[test]
    fn test_lore_history() {
        let (_tmp, repo) = setup_lore_repo();

        let (mut manifest, _) = repo
            .create_entity(
                EntityType::Character,
                "hero",
                "Test Hero",
                "integration-test",
            )
            .unwrap();

        manifest.set_property(
            "name",
            serde_yaml::Value::String("Updated Hero".to_string()),
        );
        repo.commit_manifest(&mut manifest, "update name", "integration-test", vec![])
            .unwrap();

        let hist = repo.history(EntityType::Character, "hero", 10).unwrap();
        assert!(hist.len() >= 2);
    }
}
