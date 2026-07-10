// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Deployment-independent repository API
//!
//! This module provides the unified repository API that applications use.
//! It abstracts away provider-specific details and ensures consistent behavior
//! regardless of whether the repository is hosted on Local, Portals Cloud, or Remote.

pub mod fallback;

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use tracing::info;

use super::provider::{Provider, ProviderManager, ProviderType};
use super::vcs::VcsBackend;
use super::vcs_lore::LoreBackend;

/// Repository API for deployment-independent repository operations
pub struct RepositoryApi {
    nap_home: std::path::PathBuf,
    provider_manager: ProviderManager,
}

impl RepositoryApi {
    /// Create a new repository API
    pub fn new(nap_home: &Path) -> Result<Self> {
        // Ensure NAP home directory exists with proper error handling
        std::fs::create_dir_all(nap_home).with_context(|| {
            format!(
                "Failed to create NAP home directory at '{}'. \
                    Check permissions and disk space.",
                nap_home.display()
            )
        })?;

        let mut provider_manager = ProviderManager::new(nap_home);

        // Try to load configured provider
        provider_manager
            .load_configured_provider()
            .with_context(|| {
                format!(
                    "Failed to load provider configuration from '{}'",
                    nap_home.join("provider.toml").display()
                )
            })?;

        Ok(Self {
            nap_home: nap_home.to_path_buf(),
            provider_manager,
        })
    }

    /// Initialize provider selection
    pub async fn initialize_provider_selection(&mut self) -> Result<ProviderType> {
        info!("Initializing provider selection");

        // Check if provider is already configured
        if let Some(provider) = self.provider_manager.active_provider() {
            info!("Provider already configured: {}", provider.name());
            return Ok(provider.provider_type());
        }

        // Prompt for provider selection (in real implementation, this would be interactive)
        // For now, default to Local
        let provider_type = ProviderType::Local;

        let factory = super::provider::ProviderFactory::new(&self.nap_home);
        let provider = factory.create_provider(provider_type)?;

        self.provider_manager.set_active_provider(provider.clone());
        self.provider_manager
            .save_provider_config(provider.as_ref())?;

        info!("Selected provider: {}", provider_type.as_str());
        Ok(provider_type)
    }

    /// Ensure the active provider is ready
    pub async fn ensure_provider_ready(&self) -> Result<()> {
        if let Some(provider) = self.provider_manager.active_provider() {
            provider.ensure_ready().await?;
            Ok(())
        } else {
            anyhow::bail!(
                "No active provider configured. Call initialize_provider_selection() first."
            );
        }
    }

    /// Create a new repository
    pub async fn create_repository(
        &self,
        repo_id: &str,
        workspace_id: Option<&str>,
    ) -> Result<RepositoryHandle> {
        info!("Creating repository: {}", repo_id);

        let provider = self.provider_manager.active_provider().context(
            "No active provider configured. Call initialize_provider_selection() first.",
        )?;

        // Ensure provider is ready
        provider.ensure_ready().await.with_context(|| {
            format!(
                "Failed to ensure provider '{}' is ready for repository creation",
                provider.name()
            )
        })?;

        // Get Lore URL base and workspace ID
        let lore_url_base = provider.lore_url_base().with_context(|| {
            format!(
                "Failed to get Lore URL base from provider '{}'",
                provider.name()
            )
        })?;
        let workspace = workspace_id.unwrap_or(provider.workspace_id());

        // Create Lore backend using provider configuration
        let lore_backend = LoreBackend::from_provider(&lore_url_base, workspace);

        // Create repository using Lore backend
        let workspace_root = &self.nap_home;
        let repo_path = workspace_root.join(repo_id);

        // Ensure parent directory exists
        if let Some(parent) = repo_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create parent directory for repository at '{}'",
                    repo_path.display()
                )
            })?;
        }

        // Track initial state for potential rollback
        let repo_existed_before = repo_path.exists();

        match lore_backend.init(&repo_path) {
            Ok(()) => {
                info!("Repository created: {}", repo_id);
                Ok(RepositoryHandle {
                    id: repo_id.to_string(),
                    workspace_id: workspace.to_string(),
                    path: repo_path,
                    lore_url_base,
                })
            }
            Err(e) => {
                // Rollback: remove partial repository directory if it was created during this call
                if repo_path.exists() && !repo_existed_before {
                    tracing::warn!(
                        repo_id,
                        path = %repo_path.display(),
                        "Repository creation failed, cleaning up partial repository at '{}'",
                        repo_path.display()
                    );
                    if let Err(cleanup_err) = std::fs::remove_dir_all(&repo_path) {
                        tracing::error!(
                            path = %repo_path.display(),
                            error = %cleanup_err,
                            "Failed to clean up partial repository directory after creation failure"
                        );
                    }
                }
                Err(anyhow::anyhow!(
                    "Failed to create repository '{}': {}. \
                     The operation has been rolled back and no partial state remains.",
                    repo_id,
                    e
                ))
            }
        }
    }

    /// Open an existing repository
    pub async fn open_repository(&self, repo_id: &str) -> Result<RepositoryHandle> {
        info!("Opening repository: {}", repo_id);

        let provider = self.provider_manager.active_provider().context(
            "No active provider configured. Call initialize_provider_selection() first.",
        )?;

        let lore_url_base = provider.lore_url_base().with_context(|| {
            format!(
                "Failed to get Lore URL base from provider '{}'",
                provider.name()
            )
        })?;
        let workspace_id = provider.workspace_id();

        let repo_path = self.nap_home.join(repo_id);

        if !repo_path.exists() {
            anyhow::bail!(
                "Repository '{}' not found at '{}'. \
                 Verify the repository ID and ensure it has been created.",
                repo_id,
                repo_path.display()
            );
        }

        info!("Repository opened: {}", repo_id);

        Ok(RepositoryHandle {
            id: repo_id.to_string(),
            workspace_id: workspace_id.to_string(),
            path: repo_path,
            lore_url_base,
        })
    }

    /// Publish changes (semantic operation)
    pub async fn publish(&self, repo_handle: &RepositoryHandle, message: &str) -> Result<String> {
        info!("Publishing changes to repository: {}", repo_handle.id);

        let provider = self.provider_manager.active_provider().context(
            "No active provider configured. Call initialize_provider_selection() first.",
        )?;

        let workspace_id = provider.workspace_id();
        let lore_backend = LoreBackend::from_provider(&repo_handle.lore_url_base, workspace_id);

        // Commit changes
        let commit_hash = lore_backend
            .commit(&repo_handle.path, message, "nap")
            .with_context(|| {
                format!(
                    "Failed to commit changes to repository '{}' at '{}'. \
                     Check repository state and permissions.",
                    repo_handle.id,
                    repo_handle.path.display()
                )
            })?;

        // Provider-specific publish behavior
        match provider.provider_type() {
            ProviderType::Local => {
                // Local: just commit
                info!("Published locally: {}", commit_hash);
            }
            ProviderType::PortalsCloud | ProviderType::Remote => {
                // Cloud/Remote: commit and push
                // If push fails, we need to rollback the commit to keep local state consistent
                match lore_backend.push(&repo_handle.path, None, None) {
                    Ok(_) => {
                        info!("Published and synchronized: {}", commit_hash);
                    }
                    Err(push_err) => {
                        // Rollback: revert the commit since push failed
                        tracing::warn!(
                            repo_id = %repo_handle.id,
                            commit_hash = %commit_hash,
                            "Push failed, rolling back commit to keep local state consistent"
                        );
                        let rollback_result = lore_backend.revert(&repo_handle.path, &commit_hash);
                        if let Err(revert_err) = rollback_result {
                            tracing::error!(
                                repo_id = %repo_handle.id,
                                commit_hash = %commit_hash,
                                error = %revert_err,
                                "Failed to rollback commit after push failure - repository may be in inconsistent state"
                            );
                            return Err(anyhow::anyhow!(
                                "Failed to push changes to remote for repository '{}'. \
                                 Attempted to rollback commit '{}' but failed. \
                                 Repository may be in inconsistent state. \
                                 Recovery options: \
                                 1. Manually revert the commit: lore revert {} \
                                 2. Reset to previous state: lore reset --hard HEAD~1 \
                                 3. Check repository status: lore status \
                                 Original push error: {}. Rollback error: {}",
                                repo_handle.id,
                                commit_hash,
                                commit_hash,
                                push_err,
                                revert_err
                            ));
                        }
                        return Err(anyhow::anyhow!(
                            "Failed to push changes to remote for repository '{}'. \
                             The commit has been successfully rolled back. \
                             Original error: {}",
                            repo_handle.id,
                            push_err
                        ));
                    }
                }
            }
        }

        Ok(commit_hash)
    }

    /// Get repository history
    pub async fn history(
        &self,
        repo_handle: &RepositoryHandle,
        limit: usize,
    ) -> Result<Vec<CommitInfo>> {
        info!("Getting history for repository: {}", repo_handle.id);

        let provider = self.provider_manager.active_provider().context(
            "No active provider configured. Call initialize_provider_selection() first.",
        )?;

        let workspace_id = provider.workspace_id();
        let lore_backend = LoreBackend::from_provider(&repo_handle.lore_url_base, workspace_id);

        let commits = lore_backend
            .log(&repo_handle.path, None, limit)
            .with_context(|| {
                format!(
                    "Failed to get history for repository '{}' at '{}'. \
                     Check repository state and Lore server connectivity.",
                    repo_handle.id,
                    repo_handle.path.display()
                )
            })?;

        Ok(commits)
    }

    /// Create a branch
    pub async fn create_branch(
        &self,
        repo_handle: &RepositoryHandle,
        branch_name: &str,
    ) -> Result<()> {
        info!(
            "Creating branch: {} in repository: {}",
            branch_name, repo_handle.id
        );

        let provider = self.provider_manager.active_provider().context(
            "No active provider configured. Call initialize_provider_selection() first.",
        )?;

        let workspace_id = provider.workspace_id();
        let lore_backend = LoreBackend::from_provider(&repo_handle.lore_url_base, workspace_id);

        lore_backend
            .create_branch(&repo_handle.path, branch_name)
            .with_context(|| {
                format!(
                    "Failed to create branch '{}' in repository '{}'. \
                     Verify branch name is valid and repository is in a valid state.",
                    branch_name, repo_handle.id
                )
            })?;

        Ok(())
    }

    /// Switch to a branch
    pub async fn switch_branch(
        &self,
        repo_handle: &RepositoryHandle,
        branch_name: &str,
    ) -> Result<()> {
        info!(
            "Switching to branch: {} in repository: {}",
            branch_name, repo_handle.id
        );

        let provider = self.provider_manager.active_provider().context(
            "No active provider configured. Call initialize_provider_selection() first.",
        )?;

        let workspace_id = provider.workspace_id();
        let lore_backend = LoreBackend::from_provider(&repo_handle.lore_url_base, workspace_id);

        lore_backend
            .switch_branch(&repo_handle.path, branch_name)
            .with_context(|| {
                format!(
                    "Failed to switch to branch '{}' in repository '{}'. \
                     Verify branch exists and repository is in a clean state.",
                    branch_name, repo_handle.id
                )
            })?;

        Ok(())
    }

    /// List branches
    pub async fn list_branches(&self, repo_handle: &RepositoryHandle) -> Result<Vec<String>> {
        info!("Listing branches in repository: {}", repo_handle.id);

        let provider = self.provider_manager.active_provider().context(
            "No active provider configured. Call initialize_provider_selection() first.",
        )?;

        let workspace_id = provider.workspace_id();
        let lore_backend = LoreBackend::from_provider(&repo_handle.lore_url_base, workspace_id);

        let branches = lore_backend
            .list_branches(&repo_handle.path)
            .with_context(|| {
                format!(
                    "Failed to list branches in repository '{}'. \
                     Check repository state and Lore server connectivity.",
                    repo_handle.id
                )
            })?;

        Ok(branches)
    }

    /// Synchronize repository (for cloud/remote providers)
    pub async fn sync(&self, repo_handle: &RepositoryHandle) -> Result<()> {
        info!("Synchronizing repository: {}", repo_handle.id);

        let provider = self.provider_manager.active_provider().context(
            "No active provider configured. Call initialize_provider_selection() first.",
        )?;

        match provider.provider_type() {
            ProviderType::Local => {
                info!("Synchronization not needed for local provider");
                Ok(())
            }
            ProviderType::PortalsCloud | ProviderType::Remote => {
                let workspace_id = provider.workspace_id();
                let lore_backend =
                    LoreBackend::from_provider(&repo_handle.lore_url_base, workspace_id);

                // Record current head before pull for potential rollback
                let head_before_pull = lore_backend.head_hash(&repo_handle.path).ok();

                lore_backend
                    .pull(&repo_handle.path, None, None)
                    .with_context(|| {
                        format!(
                            "Failed to pull changes for repository '{}'. \
                             Check network connectivity and remote server status.",
                            repo_handle.id
                        )
                    })?;

                match lore_backend.push(&repo_handle.path, None, None) {
                    Ok(_) => {
                        info!("Repository synchronized");
                        Ok(())
                    }
                    Err(push_err) => {
                        // Rollback: try to revert to state before pull if we recorded it
                        if let Some(ref old_head) = head_before_pull {
                            tracing::warn!(
                                repo_id = %repo_handle.id,
                                old_head = %old_head,
                                "Push failed after pull, attempting to rollback to previous state"
                            );
                            let rollback_result = lore_backend.revert(&repo_handle.path, old_head);
                            if let Err(revert_err) = rollback_result {
                                tracing::error!(
                                    repo_id = %repo_handle.id,
                                    old_head = %old_head,
                                    error = %revert_err,
                                    "Failed to rollback sync operation - repository may be in inconsistent state"
                                );
                                return Err(anyhow::anyhow!(
                                    "Failed to push changes for repository '{}'. \
                                     Pull was attempted but push failed. \
                                     Attempted to rollback to previous state '{}' but failed. \
                                     Repository may be in inconsistent state. \
                                     Recovery options: \
                                     1. Manually reset to previous state: lore reset --hard {} \
                                     2. Check repository status: lore status \
                                     3. Force sync from remote: lore pull --force \
                                     Original push error: {}. Rollback error: {}",
                                    repo_handle.id,
                                    old_head,
                                    old_head,
                                    push_err,
                                    revert_err
                                ));
                            }
                            return Err(anyhow::anyhow!(
                                "Failed to push changes for repository '{}'. \
                                 Pull was attempted but push failed. \
                                 Successfully rolled back to previous state '{}'. \
                                 Original error: {}",
                                repo_handle.id,
                                old_head,
                                push_err
                            ));
                        }
                        Err(anyhow::anyhow!(
                            "Failed to push changes for repository '{}'. \
                             Pull was attempted but push failed. \
                             Could not rollback (no previous state recorded). \
                             Original error: {}",
                            repo_handle.id,
                            push_err
                        ))
                    }
                }
            }
        }
    }

    /// Delete a repository
    pub async fn delete_repository(&self, repo_handle: &RepositoryHandle) -> Result<()> {
        info!("Deleting repository: {}", repo_handle.id);

        // Remove repository directory
        std::fs::remove_dir_all(&repo_handle.path).with_context(|| {
            format!(
                "Failed to remove repository directory at '{}'. \
                     Check file permissions and ensure no processes are using the repository.",
                repo_handle.path.display()
            )
        })?;

        info!("Repository deleted: {}", repo_handle.id);
        Ok(())
    }

    /// Get the active provider, if any.
    pub fn active_provider(&self) -> Option<&Arc<dyn Provider>> {
        self.provider_manager.active_provider()
    }

    /// Get mutable access to the provider manager (for fallback, etc.).
    pub fn provider_manager_mut(&mut self) -> &mut ProviderManager {
        &mut self.provider_manager
    }

    /// Get the provider manager (read-only).
    pub fn provider_manager(&self) -> &ProviderManager {
        &self.provider_manager
    }

    /// Get provider status
    pub async fn provider_status(&self) -> Result<super::provider::ProviderStatus> {
        let provider = self
            .provider_manager
            .active_provider()
            .context("No active provider configured")?;

        provider.status().await
    }
}

/// Handle to a repository
#[derive(Debug, Clone)]
pub struct RepositoryHandle {
    pub id: String,
    pub workspace_id: String,
    pub path: std::path::PathBuf,
    pub lore_url_base: String,
}

// Re-export CommitInfo from the vcs module (single source of truth).
// repository_api methods return this type; callers should import from here
// or from the vcs module directly.
pub use super::vcs::CommitInfo;

// Re-export fallback functionality
pub use fallback::{FallbackHandler, FallbackResult, FallbackStrategy, RepositoryApiFallback};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_repository_api_creation() {
        let temp_dir = TempDir::new().unwrap();
        let api = RepositoryApi::new(temp_dir.path()).unwrap();
        assert_eq!(api.nap_home, temp_dir.path());
    }

    #[test]
    fn test_repository_handle() {
        let handle = RepositoryHandle {
            id: "test-repo".to_string(),
            workspace_id: "default".to_string(),
            path: std::path::PathBuf::from("/tmp/test-repo"),
            lore_url_base: "lore://localhost:41337".to_string(),
        };

        assert_eq!(handle.id, "test-repo");
        assert_eq!(handle.workspace_id, "default");
    }
}
