//! Repository — filesystem layout and manifest CRUD.
//!
//! A NAP repository is a self-contained directory of entities.
//! Repository structure:
//!
//! ```text
//! toystory/               ← repository root
//! ├── repository.yaml     ← repository metadata (name, description, nap config)
//! ├── character/          ← entity type (has .entity-type marker)
//! │   ├── .entity-type    ← marker file
//! │   ├── woody.yaml
//! │   └── slinky.yaml
//! ├── location/
//! │   ├── .entity-type
//! │   └── andys-room.yaml
//! └── scene/
//!     ├── .entity-type
//!     └── pizza-planet-scene.yaml
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

/// Marker filename for entity type directories.
pub const ENTITY_TYPE_MARKER: &str = ".entity-type";

/// Sentinel returned as the "commit hash" when a write is performed in
/// unversioned mode (no version-control backend configured).
pub const UNVERSIONED_COMMIT: &str = "unversioned";

/// A NAP repository.
pub struct Repository {
    /// Filesystem path to the repository root.
    pub root: PathBuf,
    /// The repository name (derived from directory name).
    pub repository: String,
    /// The VCS backend (Lore), if a version-control backend is configured.
    /// `None` means the repository operates in unversioned (filesystem-only)
    /// mode: reads/writes work, but VCS-only operations fail informatively.
    vcs: Option<Box<dyn VcsBackend>>,
}

impl Repository {
    /// Open an existing NAP repository at the given path with a VCS backend.
    pub fn open(path: &Path, vcs: Box<dyn VcsBackend>) -> Result<Self, NapError> {
        Self::open_optional(path, Some(vcs))
    }

    /// Open an existing NAP repository at the given path.
    ///
    /// `vcs` may be `None` to operate in unversioned mode.
    pub fn open_optional(path: &Path, vcs: Option<Box<dyn VcsBackend>>) -> Result<Self, NapError> {
        // Check for repository.yaml or repository.yaml to identify valid repository
        if !path.join("repository.yaml").exists() && !path.join("repository.yaml").exists() {
            return Err(NapError::RepositoryNotFound(path.display().to_string()));
        }

        let repository = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        debug!(
            path = %path.display(),
            repository = %repository,
            "opened NAP repository"
        );

        Ok(Self {
            root: path.to_path_buf(),
            repository,
            vcs,
        })
    }

    /// Read the resolve configuration from repository.yaml (or repository.yaml) metadata.
    pub fn read_resolve_config(&self) -> ResolveConfig {
        // Prefer repository.yaml, fall back to repository.yaml
        let repo_yaml_path = self.root.join("repository.yaml");
        let universe_yaml_path = self.root.join("repository.yaml");
        let config_path = if repo_yaml_path.exists() {
            repo_yaml_path
        } else if universe_yaml_path.exists() {
            universe_yaml_path
        } else {
            debug!("no repository.yaml or repository.yaml found, using default ResolveConfig");
            return ResolveConfig::default();
        };

        let yaml_content = match std::fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(e) => {
                debug!(
                    path = %config_path.display(),
                    error = %e,
                    "failed to read config, using default ResolveConfig"
                );
                return ResolveConfig::default();
            }
        };

        // Parse YAML and extract nap metadata
        let parsed: serde_yaml::Value = match serde_yaml::from_str(&yaml_content) {
            Ok(value) => value,
            Err(e) => {
                debug!(
                    path = %config_path.display(),
                    error = %e,
                    "failed to parse config, using default ResolveConfig"
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
                path = %config_path.display(),
                "nap metadata not found, auto-creating with default_branch = 'main'"
            );

            // Read the existing manifest
            let mut manifest = match Manifest::from_file(&config_path) {
                Ok(m) => m,
                Err(e) => {
                    debug!(
                        path = %config_path.display(),
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
            if let Err(e) = manifest.to_file(&config_path) {
                debug!(
                    path = %config_path.display(),
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

    /// Initialize a new NAP repository with a VCS backend.
    pub fn init(path: &Path, repository: &str, vcs: Box<dyn VcsBackend>) -> Result<Self, NapError> {
        Self::init_optional(path, repository, Some(vcs))
    }

    /// Initialize a new NAP repository.
    ///
    /// `vcs` may be `None` to initialize in unversioned mode — the repository
    /// structure is created and an initial filesystem state is written, but no
    /// version-control workspace or initial commit is created.
    pub fn init_optional(
        path: &Path,
        repository: &str,
        vcs: Option<Box<dyn VcsBackend>>,
    ) -> Result<Self, NapError> {
        let repo_root = path.to_path_buf();
        if repo_root.join("repository.yaml").exists() || repo_root.join("repository.yaml").exists()
        {
            return Err(NapError::RepositoryAlreadyExists(
                repo_root.display().to_string(),
            ));
        }

        info!(
            path = %repo_root.display(),
            repository = %repository,
            "initializing NAP repository"
        );

        // Create directory structure
        std::fs::create_dir_all(&repo_root)?;

        // Create repository.yaml with [nap] metadata
        let mut repo_manifest = Manifest::new(
            repository,
            EntityType::new("world"),
            repository,
            &format!("{repository} Repository"),
        );

        // Add nap configuration to metadata
        repo_manifest.metadata.insert(
            "nap".to_string(),
            serde_yaml::to_value(serde_json::json!({
                "default_branch": "main"
            }))
            .unwrap(),
        );

        repo_manifest.to_file(&repo_root.join("repository.yaml"))?;

        // Initialize VCS + initial commit when a backend is configured
        if let Some(vcs) = vcs.as_ref() {
            vcs.init(&repo_root)?;
            vcs.commit(
                &repo_root,
                &format!("Initialize {repository} repository"),
                "nap-init",
            )?;
        }

        info!(
            path = %repo_root.display(),
            repository = %repository,
            "NAP repository initialized successfully"
        );

        Ok(Self {
            root: repo_root,
            repository: repository.to_string(),
            vcs,
        })
    }

    /// Bootstrap a VCS backend for an already-initialized repository that was
    /// created in unversioned mode.
    ///
    /// Initializes the backend at the repository root and creates an initial
    /// commit capturing the current filesystem state. Used by
    /// `nap backend configure` when a backend is configured after the fact.
    ///
    /// Errors if no backend is available (unversioned mode).
    pub fn bootstrap_vcs(&self, message: &str, author: &str) -> Result<String, NapError> {
        let vcs = self.require_vcs("bootstrap the repository into version control")?;
        vcs.init(&self.root)?;
        let hash = vcs.commit(&self.root, message, author)?;
        info!(
            repository = %self.repository,
            commit_hash = %hash,
            "bootstrapped repository into version control"
        );
        Ok(hash)
    }

    fn require_vcs(&self, operation: &str) -> Result<&dyn VcsBackend, NapError> {
        self.vcs
            .as_deref()
            .ok_or_else(|| NapError::BackendNotConfigured {
                operation: operation.to_string(),
            })
    }

    /// Get the full filesystem path to an entity's manifest file.
    pub fn manifest_path(&self, entity_type: &EntityType, entity_id: &str) -> PathBuf {
        let uri = NapUri::new(&self.repository, entity_type.clone(), entity_id);
        self.root.join(uri.manifest_path())
    }

    /// Read a manifest from the repository.
    pub fn read_manifest(
        &self,
        entity_type: &EntityType,
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
        entity_type: &EntityType,
        entity_id: &str,
        reference: &str,
    ) -> Result<Manifest, NapError> {
        let uri = NapUri::new(&self.repository, entity_type.clone(), entity_id);
        let file_path = uri.manifest_path();

        debug!(
            file_path = %file_path,
            reference = %reference,
            "reading manifest at ref"
        );

        let content = self.require_vcs("read manifest at ref")?.read_file_at_ref(
            &self.root,
            &file_path,
            Some(reference),
        )?;
        Manifest::from_yaml(&content)
    }

    /// Write a manifest to the repository (does NOT commit).
    pub fn write_manifest(&self, manifest: &Manifest) -> Result<PathBuf, NapError> {
        let uri: NapUri = manifest.id.parse()?;
        let entity_type = uri.entity_type.clone();

        // Ensure the entity type directory and marker exist
        self.ensure_entity_type_dir(&entity_type)?;

        let path = self.root.join(uri.manifest_path());

        debug!(
            path = %path.display(),
            manifest_id = %manifest.id,
            "writing manifest"
        );

        manifest.to_file(&path)?;
        Ok(path)
    }

    /// Ensure an entity type directory exists with its marker file.
    fn ensure_entity_type_dir(&self, entity_type: &EntityType) -> Result<(), NapError> {
        let dir = self.root.join(entity_type.directory_name());
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
            // Create .entity-type marker file
            let marker = dir.join(ENTITY_TYPE_MARKER);
            std::fs::write(&marker, "")?;
            debug!(
                entity_type = %entity_type,
                path = %dir.display(),
                "created entity type directory with marker"
            );
        }
        Ok(())
    }

    /// Create a new entity manifest and commit it.
    pub fn create_entity(
        &self,
        entity_type: &EntityType,
        entity_id: &str,
        name: &str,
        author: &str,
    ) -> Result<(Manifest, String), NapError> {
        // Ensure entity type directory exists
        self.ensure_entity_type_dir(entity_type)?;

        let mut manifest = Manifest::new(&self.repository, entity_type.clone(), entity_id, name);

        // Check if entity already exists (idempotency guard)
        let path = self.manifest_path(entity_type, entity_id);
        if path.exists() {
            return Err(NapError::Other(format!(
                "entity '{entity_id}' of type '{entity_type}' already exists"
            )));
        }

        manifest.bump_version();

        // Validate against schema before writing
        crate::schema::validate_manifest(&manifest)
            .map_err(|errors| NapError::ManifestValidationError(errors.join("; ")))?;

        // Write the manifest
        self.write_manifest(&manifest)?;

        // Commit via VCS when a backend is configured; otherwise the write
        // above is the only durable change (unversioned mode).
        let commit_hash = match self.vcs.as_ref() {
            Some(vcs) => {
                let commit_message = format!("Create {entity_type} '{name}' ({entity_id})");
                vcs.commit(&self.root, &commit_message, author)?
            }
            None => UNVERSIONED_COMMIT.to_string(),
        };

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

        // Write updated manifest.
        self.write_manifest(manifest)?;

        // Compute manifest hash
        let manifest_hash = manifest.content_hash()?.as_str().to_string();

        // VCS commit — produces the new HEAD hash. When no backend is
        // configured (unversioned mode), there is no history to record and the
        // filesystem write above is the only durable change.
        let (parent, vcs_hash) = match self.vcs.as_ref() {
            Some(vcs) => (
                Some(vcs.head_hash(&self.root)?),
                vcs.commit(&self.root, message, author)?,
            ),
            None => (None, UNVERSIONED_COMMIT.to_string()),
        };

        // Create NAP commit metadata with the previous VCS HEAD as parent.
        let nap_commit = Commit::new(parent, author, message, &manifest_hash, changes);

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
        entity_type: &EntityType,
        entity_id: &str,
        limit: usize,
    ) -> Result<Vec<crate::vcs::CommitInfo>, NapError> {
        let uri = NapUri::new(&self.repository, entity_type.clone(), entity_id);
        let file_path = uri.manifest_path();
        self.require_vcs("view history")?
            .log(&self.root, Some(&file_path), limit)
    }

    /// List all entity IDs of a given type in the repository.
    pub fn list_entities(&self, entity_type: &EntityType) -> Result<Vec<String>, NapError> {
        let dir = self.root.join(entity_type.directory_name());
        if !dir.exists() {
            return Err(NapError::EntityTypeNotFound(entity_type.to_string()));
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

    /// Discover all entity types in this repository.
    ///
    /// Scans the repository root for directories containing a `.entity-type`
    /// marker file OR directories containing at least one `.yaml` file
    /// (implicit discovery for backward compatibility).
    pub fn list_entity_types(&self) -> Result<Vec<EntityType>, NapError> {
        let mut types = Vec::new();
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            // Skip hidden directories (.nap, etc.)
            let dir_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name,
                None => continue,
            };
            if dir_name.starts_with('.') || dir_name == "target" {
                continue;
            }

            // Check for .entity-type marker file (explicit)
            if path.join(ENTITY_TYPE_MARKER).exists() {
                types.push(EntityType::new(dir_name));
                continue;
            }

            // Implicit: directory contains at least one .yaml file
            let has_yaml = std::fs::read_dir(&path)
                .ok()
                .and_then(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .find(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("yaml"))
                        .map(|_| true)
                })
                .unwrap_or(false);

            if has_yaml {
                types.push(EntityType::new(dir_name));
            }
        }
        types.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        Ok(types)
    }

    /// Delete an entity manifest and commit the deletion.
    pub fn delete_entity(
        &self,
        entity_type: &EntityType,
        entity_id: &str,
        author: &str,
    ) -> Result<String, NapError> {
        let path = self.manifest_path(entity_type, entity_id);
        if !path.exists() {
            return Err(NapError::ManifestNotFound(path.display().to_string()));
        }

        std::fs::remove_file(&path)?;

        let message = format!("Delete {entity_type} '{entity_id}'");
        let hash = match self.vcs.as_ref() {
            Some(vcs) => vcs.commit(&self.root, &message, author)?,
            None => UNVERSIONED_COMMIT.to_string(),
        };
        info!(entity_type = %entity_type, entity_id = %entity_id, "deleted entity");
        Ok(hash)
    }

    /// Create a branch in the underlying VCS.
    pub fn create_branch(&self, name: &str) -> Result<(), NapError> {
        self.require_vcs("create branch")?
            .create_branch(&self.root, name)
    }

    /// Switch to a branch.
    pub fn switch_branch(&self, name: &str) -> Result<(), NapError> {
        self.require_vcs("switch branch")?
            .switch_branch(&self.root, name)
    }

    /// List branches.
    pub fn list_branches(&self) -> Result<Vec<String>, NapError> {
        self.require_vcs("list branches")?.list_branches(&self.root)
    }

    /// Revert a commit by creating a new VCS commit that undoes the specified one.
    pub fn revert_commit(&self, commit_hash: &str, author: &str) -> Result<String, NapError> {
        let vcs = self.require_vcs("revert commit")?;
        let new_hash = vcs.revert(&self.root, commit_hash)?;

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
        self.require_vcs("read HEAD hash")?.head_hash(&self.root)
    }

    /// Resolve the most recent commit hash on a given branch.
    pub fn resolve_branch_head(&self, branch: &str) -> Result<String, NapError> {
        self.require_vcs("resolve branch head")?
            .resolve_branch_head(&self.root, branch)
    }

    // ── Remote operations ─────────────────────────────────────────

    /// Add a remote to the repository.
    pub fn add_remote(&self, name: &str, url: &str) -> Result<(), NapError> {
        self.require_vcs("add remote")?
            .add_remote(&self.root, name, url)
    }

    /// Remove a remote from the repository.
    pub fn remove_remote(&self, name: &str) -> Result<(), NapError> {
        self.require_vcs("remove remote")?
            .remove_remote(&self.root, name)
    }

    /// List remotes as `(name, url)` pairs.
    pub fn list_remotes(&self) -> Result<Vec<(String, String)>, NapError> {
        self.require_vcs("list remotes")?.list_remotes(&self.root)
    }

    /// Push the current branch to a remote.
    pub fn push(&self, remote: Option<&str>, branch: Option<&str>) -> Result<(), NapError> {
        self.require_vcs("push")?.push(&self.root, remote, branch)
    }

    /// Pull the current branch from a remote.
    pub fn pull(&self, remote: Option<&str>, branch: Option<&str>) -> Result<(), NapError> {
        self.require_vcs("pull")?.pull(&self.root, remote, branch)
    }

    /// Access the VCS backend (for the resolver to read files at specific refs).
    ///
    /// Returns `None` when no version-control backend is configured (unversioned mode).
    pub fn vcs(&self) -> Option<&dyn VcsBackend> {
        self.vcs.as_deref()
    }
}

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

        assert!(repo.root.join("repository.yaml").exists());
    }

    #[test]
    fn test_create_and_read_entity() {
        let tmp = TempDir::new().unwrap();
        let repo = mock_repo(&tmp);

        let (manifest, hash) = repo
            .create_entity(
                &EntityType::new("character"),
                "hero",
                "The Hero",
                "test-author",
            )
            .unwrap();

        assert_eq!(manifest.name, "The Hero");
        assert_eq!(manifest.entity_type.as_str(), "character");
        assert_eq!(manifest.version, 1);
        assert_eq!(repo.head_hash().unwrap(), hash);

        // Read it back
        let read_back = repo
            .read_manifest(&EntityType::new("character"), "hero")
            .unwrap();
        assert_eq!(read_back.name, "The Hero");
        assert_eq!(read_back.version, 1);
    }

    #[test]
    fn test_create_entity_auto_creates_type_directory() {
        let tmp = TempDir::new().unwrap();
        let repo = mock_repo(&tmp);

        // Create entity of a custom type
        repo.create_entity(&EntityType::new("pokemon"), "pikachu", "Pikachu", "test")
            .unwrap();

        // Verify the type directory and marker exist
        assert!(repo.root.join("pokemon").exists());
        assert!(repo.root.join("pokemon").join(".entity-type").exists());
        assert!(repo.root.join("pokemon/pikachu.yaml").exists());
    }

    #[test]
    fn test_list_entity_types() {
        let tmp = TempDir::new().unwrap();
        let repo = mock_repo(&tmp);

        // Create entities of different types
        repo.create_entity(&EntityType::new("character"), "hero", "Hero", "author")
            .unwrap();
        repo.create_entity(&EntityType::new("location"), "village", "Village", "author")
            .unwrap();
        repo.create_entity(&EntityType::new("pokemon"), "pikachu", "Pikachu", "author")
            .unwrap();

        let types = repo.list_entity_types().unwrap();
        assert!(types.contains(&EntityType::new("character")));
        assert!(types.contains(&EntityType::new("location")));
        assert!(types.contains(&EntityType::new("pokemon")));
    }

    #[test]
    fn test_list_entities() {
        let tmp = TempDir::new().unwrap();
        let repo = mock_repo(&tmp);

        repo.create_entity(&EntityType::new("character"), "alice", "Alice", "author")
            .unwrap();
        repo.create_entity(&EntityType::new("character"), "bob", "Bob", "author")
            .unwrap();

        let chars = repo.list_entities(&EntityType::new("character")).unwrap();
        assert_eq!(chars, vec!["alice", "bob"]);
    }

    #[test]
    fn test_commit_manifest_updates() {
        let tmp = TempDir::new().unwrap();
        let repo = mock_repo(&tmp);

        let (mut manifest, create_hash) = repo
            .create_entity(&EntityType::new("character"), "hero", "The Hero", "author")
            .unwrap();

        // Modify and commit
        manifest.set_property("toy_type", serde_yaml::Value::String("elf".to_string()));
        let changes = vec![Change::set("properties.toy_type", None, "elf".to_string())];
        let commit = repo
            .commit_manifest(&mut manifest, "set toy_type to elf", "author", changes)
            .unwrap();

        assert!(!commit.id.is_empty());
        assert_eq!(commit.message, "set toy_type to elf");
        assert_eq!(commit.parent.as_deref(), Some(create_hash.as_str()));

        // Verify version incremented
        let read_back = repo
            .read_manifest(&EntityType::new("character"), "hero")
            .unwrap();
        assert!(read_back.version >= 2);
    }

    #[test]
    fn test_history() {
        let tmp = TempDir::new().unwrap();
        let repo = mock_repo(&tmp);

        let (mut manifest, _) = repo
            .create_entity(&EntityType::new("character"), "hero", "The Hero", "author")
            .unwrap();

        manifest.set_property(
            "name",
            serde_yaml::Value::String("Updated Hero".to_string()),
        );
        repo.commit_manifest(&mut manifest, "update name", "author", vec![])
            .unwrap();

        let hist = repo
            .history(&EntityType::new("character"), "hero", 10)
            .unwrap();
        assert!(hist.len() >= 2);
    }

    #[test]
    fn test_revert_commit() {
        let tmp = TempDir::new().unwrap();
        let repo = mock_repo(&tmp);

        // Create entity and note its name
        let (mut manifest, _) = repo
            .create_entity(&EntityType::new("character"), "hero", "The Hero", "author")
            .unwrap();
        assert_eq!(manifest.name, "The Hero");

        // Modify and commit
        manifest.set_property("toy_type", serde_yaml::Value::String("elf".to_string()));
        let changes = vec![Change::set("properties.toy_type", None, "elf".to_string())];
        repo.commit_manifest(&mut manifest, "set toy_type to elf", "author", changes)
            .unwrap();
        let update_hash = repo.head_hash().unwrap();

        // Revert the VCS commit.
        let revert_hash = repo.revert_commit(&update_hash, "author").unwrap();
        assert!(!revert_hash.is_empty());

        // Verify the manifest remains normal YAML without a cached head field.
        let manifest_path = repo.manifest_path(&EntityType::new("character"), "hero");
        let manifest_yaml = std::fs::read_to_string(&manifest_path).unwrap();
        assert!(!manifest_yaml.contains("\nhead:"));

        // Verify the revert appears in history
        let hist = repo
            .history(&EntityType::new("character"), "hero", 10)
            .unwrap();
        assert!(hist.iter().any(|c| c.id == revert_hash));
    }

    // ── Unversioned mode ────────────────────────────────────────────────

    fn unversioned_repo(tmp: &TempDir) -> Repository {
        let repo_path = tmp.path().join("testverse");
        Repository::init_optional(&repo_path, "testverse", None).unwrap()
    }

    #[test]
    fn test_unversioned_init_creates_structure() {
        let tmp = TempDir::new().unwrap();
        let repo = unversioned_repo(&tmp);

        assert!(repo.root.join("repository.yaml").exists());
        // No backend was provided, so VCS is absent.
        assert!(repo.vcs().is_none());
    }

    #[test]
    fn test_unversioned_create_returns_sentinel_commit() {
        let tmp = TempDir::new().unwrap();
        let repo = unversioned_repo(&tmp);

        let (manifest, hash) = repo
            .create_entity(
                &EntityType::new("character"),
                "hero",
                "The Hero",
                "test-author",
            )
            .unwrap();

        assert_eq!(manifest.name, "The Hero");
        assert_eq!(hash, UNVERSIONED_COMMIT);
        assert!(repo.root.join("character/hero.yaml").exists());

        // The file is durable even without a backend.
        let read_back = repo
            .read_manifest(&EntityType::new("character"), "hero")
            .unwrap();
        assert_eq!(read_back.name, "The Hero");
    }

    #[test]
    fn test_unversioned_commit_manifest_persists() {
        let tmp = TempDir::new().unwrap();
        let repo = unversioned_repo(&tmp);

        let (mut manifest, _) = repo
            .create_entity(&EntityType::new("character"), "hero", "The Hero", "author")
            .unwrap();

        manifest.set_property("toy_type", serde_yaml::Value::String("elf".to_string()));
        let changes = vec![Change::set("properties.toy_type", None, "elf".to_string())];
        let commit = repo
            .commit_manifest(&mut manifest, "set toy_type to elf", "author", changes)
            .unwrap();

        // No VCS history exists, so the commit has no parent.
        assert!(commit.parent.is_none());

        let read_back = repo
            .read_manifest(&EntityType::new("character"), "hero")
            .unwrap();
        assert_eq!(read_back.properties["toy_type"], "elf");
    }

    #[test]
    fn test_unversioned_vcs_operations_error() {
        let tmp = TempDir::new().unwrap();
        let repo = unversioned_repo(&tmp);

        repo.create_entity(&EntityType::new("character"), "hero", "The Hero", "author")
            .unwrap();

        // VCS-only operations fail with a BackendNotConfigured error.
        let err = repo
            .history(&EntityType::new("character"), "hero", 10)
            .unwrap_err();
        assert!(matches!(err, NapError::BackendNotConfigured { .. }));

        let err = repo.list_branches().unwrap_err();
        assert!(matches!(err, NapError::BackendNotConfigured { .. }));

        let err = repo.head_hash().unwrap_err();
        assert!(matches!(err, NapError::BackendNotConfigured { .. }));

        let err = repo.push(Some("origin"), None).unwrap_err();
        assert!(matches!(err, NapError::BackendNotConfigured { .. }));

        // The error message guides the user toward configuration.
        let msg = err.to_string();
        assert!(msg.contains("nap backend configure"));
    }

    #[test]
    fn test_bootstrap_vcs_attaches_backend_and_commits() {
        let tmp = TempDir::new().unwrap();
        let unversioned = unversioned_repo(&tmp);

        // Seed some filesystem state while unversioned.
        unversioned
            .create_entity(&EntityType::new("character"), "hero", "The Hero", "author")
            .unwrap();

        // Attach a backend and bootstrap the current state as the baseline.
        let backend = MockBackend::new();
        let bootstrapped =
            Repository::open_optional(&unversioned.root, Some(Box::new(backend))).unwrap();

        let hash = bootstrapped
            .bootstrap_vcs("Initialize existing NAP repository", "nap")
            .unwrap();

        assert!(!hash.is_empty());
        assert!(bootstrapped.head_hash().unwrap() == hash);
    }

    #[test]
    fn test_bootstrap_vcs_without_backend_errors() {
        let tmp = TempDir::new().unwrap();
        let repo = unversioned_repo(&tmp);

        let err = repo
            .bootstrap_vcs("Initialize existing NAP repository", "nap")
            .unwrap_err();
        assert!(matches!(err, NapError::BackendNotConfigured { .. }));
        let msg = err.to_string();
        assert!(msg.contains("nap backend configure"));
    }
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
        let repository = format!("ri-{}", unique_suffix());
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join(&repository);
        let repo =
            Repository::init(&repo_path, &repository, Box::new(LoreBackend::from_env())).unwrap();
        (tmp, repo)
    }

    #[test]
    fn test_lore_init_creates_structure() {
        let (_tmp, repo) = setup_lore_repo();
        assert!(repo.root.join(".nap").exists());
        assert!(repo.root.join("repository.yaml").exists());
    }

    #[test]
    fn test_lore_create_and_read_entity() {
        let (_tmp, repo) = setup_lore_repo();

        let (manifest, _hash) = repo
            .create_entity(
                &EntityType::new("character"),
                "hero",
                "Test Hero",
                "integration-test",
            )
            .unwrap();
        assert_eq!(manifest.name, "Test Hero");

        let read_back = repo
            .read_manifest(&EntityType::new("character"), "hero")
            .unwrap();
        assert_eq!(read_back.name, "Test Hero");
    }

    #[test]
    fn test_lore_commit_and_branch() {
        let (_tmp, repo) = setup_lore_repo();

        let (mut manifest, _) = repo
            .create_entity(
                &EntityType::new("character"),
                "hero",
                "Test Hero",
                "integration-test",
            )
            .unwrap();

        manifest.set_property("toy_type", serde_yaml::Value::String("plush".to_string()));
        let changes = vec![Change::set(
            "properties.toy_type",
            None,
            "plush".to_string(),
        )];
        repo.commit_manifest(&mut manifest, "add toy_type", "integration-test", changes)
            .unwrap();

        let read_back = repo
            .read_manifest(&EntityType::new("character"), "hero")
            .unwrap();
        assert_eq!(read_back.version, 2);

        repo.create_branch("feature-branch").unwrap();
        let branches = repo.list_branches().unwrap();
        assert!(branches.contains(&"feature-branch".to_string()));
    }

    #[test]
    fn test_lore_delete_entity() {
        let (_tmp, repo) = setup_lore_repo();

        repo.create_entity(
            &EntityType::new("character"),
            "hero",
            "Test Hero",
            "integration-test",
        )
        .unwrap();

        repo.delete_entity(&EntityType::new("character"), "hero", "integration-test")
            .unwrap();

        let entities = repo.list_entities(&EntityType::new("character")).unwrap();
        assert!(!entities.contains(&"hero".to_string()));
    }

    #[test]
    fn test_lore_history() {
        let (_tmp, repo) = setup_lore_repo();

        let (mut manifest, _) = repo
            .create_entity(
                &EntityType::new("character"),
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

        let hist = repo
            .history(&EntityType::new("character"), "hero", 10)
            .unwrap();
        assert!(hist.len() >= 2);
    }
}
