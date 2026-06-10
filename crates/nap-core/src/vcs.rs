//! VCS backend trait — abstraction over version control systems.
//!
//! NAP's v0 uses Git (via gitoxide/gix). The `VcsBackend` trait abstracts
//! the VCS so that Fossil, Jujutsu, or other backends can be swapped in
//! without changing the rest of the protocol implementation.
//!
//! The trait surface is deliberately minimal — only the operations NAP
//! actually needs. No attempt to model every VCS feature.

use std::path::Path;

use crate::error::NapError;

/// Metadata about a single VCS commit, returned by `log()`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommitInfo {
    /// The commit hash/identifier.
    pub id: String,
    /// Parent commit hash (None for root).
    pub parent: Option<String>,
    /// Commit author.
    pub author: String,
    /// Commit message.
    pub message: String,
    /// Commit timestamp (RFC 3339).
    pub timestamp: String,
}

/// Abstraction over a distributed version control system.
///
/// Implementors: [`GitBackend`](crate::vcs_git::GitBackend), future
/// `FossilBackend`, `JjBackend`, etc.
pub trait VcsBackend: Send + Sync {
    /// Initialize a new repository at the given path.
    fn init(&self, path: &Path) -> Result<(), NapError>;

    /// Stage all files and create a commit.
    fn commit(&self, path: &Path, message: &str, author: &str) -> Result<String, NapError>;

    /// Read a file's content at a specific ref (branch, tag, or commit hash).
    /// If `reference` is None, reads from the current working tree.
    fn read_file_at_ref(
        &self,
        repo_path: &Path,
        file_path: &str,
        reference: Option<&str>,
    ) -> Result<String, NapError>;

    /// Get the commit log for the repository, optionally filtered to a specific file.
    fn log(
        &self,
        path: &Path,
        file: Option<&str>,
        limit: usize,
    ) -> Result<Vec<CommitInfo>, NapError>;

    /// Create a new branch.
    fn create_branch(&self, path: &Path, name: &str) -> Result<(), NapError>;

    /// Switch to a branch.
    fn switch_branch(&self, path: &Path, name: &str) -> Result<(), NapError>;

    /// Create a tag at the current HEAD.
    fn create_tag(&self, path: &Path, name: &str) -> Result<(), NapError>;

    /// Get the current branch name.
    fn current_branch(&self, path: &Path) -> Result<String, NapError>;

    /// Get the HEAD commit hash.
    fn head_hash(&self, path: &Path) -> Result<String, NapError>;

    /// Revert a commit by creating a new commit that undoes it.
    ///
    /// The default implementation returns an error — not all VCS backends
    /// support revert. Git does via `git revert --no-edit`.
    /// Returns the SHA of the new revert commit.
    fn revert(&self, _path: &Path, _commit_hash: &str) -> Result<String, NapError> {
        Err(NapError::VcsError(
            "revert not supported by this VCS backend".to_string(),
        ))
    }

    /// List all branches.
    fn list_branches(&self, path: &Path) -> Result<Vec<String>, NapError>;

    /// List all tags.
    fn list_tags(&self, path: &Path) -> Result<Vec<String>, NapError>;

    // ── Remote operations ────────────────────────────────────────────

    /// Add a remote.
    fn add_remote(&self, path: &Path, name: &str, url: &str) -> Result<(), NapError>;

    /// Remove a remote.
    fn remove_remote(&self, path: &Path, name: &str) -> Result<(), NapError>;

    /// List remotes as `(name, url)` pairs.
    fn list_remotes(&self, path: &Path) -> Result<Vec<(String, String)>, NapError>;

    /// Push the current branch to its upstream / a named remote.
    ///
    /// Delegates to `git push <remote> <branch>` or, if both are omitted,
    /// to `git push` (which uses the tracking-branch configuration).
    fn push(&self, path: &Path, remote: Option<&str>, branch: Option<&str>)
    -> Result<(), NapError>;

    /// Pull the current branch from its upstream / a named remote.
    ///
    /// Delegates to `git pull <remote> <branch>` or, if both are omitted,
    /// to `git pull` (which uses the tracking-branch configuration).
    fn pull(&self, path: &Path, remote: Option<&str>, branch: Option<&str>)
    -> Result<(), NapError>;
}
