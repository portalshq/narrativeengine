use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use nap_core::error::NapError;
use nap_core::vcs::{CommitInfo, VcsBackend};

/// A VcsBackend that simulates VCS operations in memory without shelling
/// out. Used by the Repository unit tests so they don't require a real
/// `lore` binary.
pub struct MockBackend {
    /// Incrementing counter for commit hashes.
    counter: AtomicU64,
    /// Stored commit metadata keyed by "hash".
    commits: Mutex<Vec<CommitInfo>>,
    /// Remote tracking.
    remotes: Mutex<HashMap<String, String>>,
    /// Branches.
    branches: Mutex<Vec<String>>,
    /// Current branch.
    current_branch: Mutex<String>,
    /// Tags.
    tags: Mutex<Vec<String>>,
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBackend {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(1),
            commits: Mutex::new(Vec::new()),
            remotes: Mutex::new(HashMap::new()),
            branches: Mutex::new(vec!["main".to_string()]),
            current_branch: Mutex::new("main".to_string()),
            tags: Mutex::new(Vec::new()),
        }
    }
}

impl VcsBackend for MockBackend {
    fn init(&self, _path: &Path) -> Result<(), NapError> {
        Ok(())
    }

    fn commit(&self, _path: &Path, message: &str, author: &str) -> Result<String, NapError> {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        // Use a valid 40-char hex hash so manifest schema validation passes.
        let hash = format!("{:064x}", n);
        let mut commits = self.commits.lock().unwrap();
        let parent = commits.last().map(|c| c.id.clone());
        let info = CommitInfo {
            id: hash.clone(),
            parent,
            author: author.to_string(),
            message: message.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        commits.push(info);
        Ok(hash)
    }

    fn read_file_at_ref(
        &self,
        repo_path: &Path,
        file_path: &str,
        _reference: Option<&str>,
    ) -> Result<String, NapError> {
        let full_path = repo_path.join(file_path);
        std::fs::read_to_string(&full_path)
            .map_err(|e| NapError::Other(format!("mock: read failed: {}", e)))
    }

    fn log(
        &self,
        _path: &Path,
        _file: Option<&str>,
        _limit: usize,
    ) -> Result<Vec<CommitInfo>, NapError> {
        let commits = self.commits.lock().unwrap();
        Ok(commits.clone())
    }

    fn create_branch(&self, _path: &Path, name: &str) -> Result<(), NapError> {
        self.branches.lock().unwrap().push(name.to_string());
        Ok(())
    }

    fn switch_branch(&self, _path: &Path, name: &str) -> Result<(), NapError> {
        *self.current_branch.lock().unwrap() = name.to_string();
        Ok(())
    }

    fn create_tag(&self, _path: &Path, name: &str) -> Result<(), NapError> {
        let mut tags = self.tags.lock().unwrap();
        if !tags.contains(&name.to_string()) {
            tags.push(name.to_string());
        }
        Ok(())
    }

    fn current_branch(&self, _path: &Path) -> Result<String, NapError> {
        Ok(self.current_branch.lock().unwrap().clone())
    }

    fn head_hash(&self, _path: &Path) -> Result<String, NapError> {
        let commits = self.commits.lock().unwrap();
        commits
            .last()
            .map(|c| c.id.clone())
            .ok_or_else(|| NapError::VcsError("no commits".to_string()))
    }

    fn resolve_branch_head(&self, _path: &Path, _branch: &str) -> Result<String, NapError> {
        let commits = self.commits.lock().unwrap();
        commits
            .last()
            .map(|c| c.id.clone())
            .ok_or_else(|| NapError::VcsError("no commits on branch".to_string()))
    }

    fn revert(&self, _repo_path: &Path, commit_hash: &str) -> Result<String, NapError> {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        let hash = format!("{:064x}", n);
        let info = CommitInfo {
            id: hash.clone(),
            parent: Some(commit_hash.to_string()),
            author: "revert".to_string(),
            message: format!("revert {}", commit_hash),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.commits.lock().unwrap().push(info);
        Ok(hash)
    }

    fn list_branches(&self, _path: &Path) -> Result<Vec<String>, NapError> {
        Ok(self.branches.lock().unwrap().clone())
    }

    fn list_tags(&self, _path: &Path) -> Result<Vec<String>, NapError> {
        Ok(self.tags.lock().unwrap().clone())
    }

    fn add_remote(&self, _path: &Path, name: &str, url: &str) -> Result<(), NapError> {
        self.remotes
            .lock()
            .unwrap()
            .insert(name.to_string(), url.to_string());
        Ok(())
    }

    fn remove_remote(&self, _path: &Path, name: &str) -> Result<(), NapError> {
        self.remotes.lock().unwrap().remove(name);
        Ok(())
    }

    fn list_remotes(&self, _path: &Path) -> Result<Vec<(String, String)>, NapError> {
        let remotes = self.remotes.lock().unwrap();
        Ok(remotes
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }

    fn push(
        &self,
        _path: &Path,
        _remote: Option<&str>,
        _branch: Option<&str>,
    ) -> Result<(), NapError> {
        Ok(())
    }

    fn pull(
        &self,
        _path: &Path,
        _remote: Option<&str>,
        _branch: Option<&str>,
    ) -> Result<(), NapError> {
        Ok(())
    }

    fn remote_url_base(&self) -> Result<String, NapError> {
        Ok("lore://localhost:8700".to_string())
    }
}
