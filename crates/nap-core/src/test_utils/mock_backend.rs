use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::NapError;
use crate::vcs::{CommitInfo, VcsBackend};

/// A VcsBackend that simulates VCS operations on the filesystem.
pub struct MockBackend {
    /// Incrementing counter for commit hashes.
    counter: AtomicU64,
    branches: Mutex<Vec<String>>,
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
            branches: Mutex::new(vec!["main".to_string()]),
        }
    }

    fn commits_path(&self, repo_path: &Path) -> PathBuf {
        repo_path.join(".mock_commits.json")
    }

    fn file_metadata_path(&self, repo_path: &Path) -> PathBuf {
        repo_path.join(".mock_file_metadata.json")
    }

    fn provenance_blobs_path(&self, repo_path: &Path) -> PathBuf {
        repo_path.join(".mock_provenance_blobs.json")
    }

    fn metadata_requests_path(&self, repo_path: &Path) -> PathBuf {
        repo_path.join(".mock_metadata_requests.json")
    }

    fn load_commits(&self, repo_path: &Path) -> Vec<CommitInfo> {
        let path = self.commits_path(repo_path);
        if !path.exists() {
            return Vec::new();
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    }

    fn save_commits(&self, repo_path: &Path, commits: &[CommitInfo]) {
        let path = self.commits_path(repo_path);
        let content = serde_json::to_string(commits).unwrap();
        std::fs::write(path, content).unwrap();
    }
}

impl VcsBackend for MockBackend {
    fn init(&self, _path: &Path) -> Result<(), NapError> {
        Ok(())
    }

    fn commit(&self, repo_path: &Path, message: &str, author: &str) -> Result<String, NapError> {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        // Use a valid 40-char hex hash so manifest schema validation passes.
        let hash = format!("{:064x}", n);
        let mut commits = self.load_commits(repo_path);
        let parent = commits.last().map(|c| c.id.clone());
        let info = CommitInfo {
            id: hash.clone(),
            parent,
            author: author.to_string(),
            message: message.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        commits.push(info);
        self.save_commits(repo_path, &commits);
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

    fn file_metadata_at_ref(
        &self,
        repo_path: &Path,
        file_path: &str,
        reference: &str,
    ) -> Result<Option<BTreeMap<String, String>>, NapError> {
        let requests_path = self.metadata_requests_path(repo_path);
        let mut requests: Vec<BTreeMap<String, String>> = if requests_path.exists() {
            std::fs::read_to_string(&requests_path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        requests.push(BTreeMap::from([
            ("path".to_string(), file_path.to_string()),
            ("revision".to_string(), reference.to_string()),
        ]));
        std::fs::write(&requests_path, serde_json::to_string(&requests).unwrap())?;

        let path = self.file_metadata_path(repo_path);
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(path)?;
        let metadata: BTreeMap<String, BTreeMap<String, String>> =
            serde_json::from_str(&content).unwrap_or_default();
        Ok(metadata
            .get(&format!("{reference}:{file_path}"))
            .cloned()
            .or_else(|| metadata.get(file_path).cloned()))
    }

    fn read_provenance_blob(&self, repo_path: &Path, address: &str) -> Result<String, NapError> {
        let path = self.provenance_blobs_path(repo_path);
        let content = std::fs::read_to_string(path)?;
        let blobs: BTreeMap<String, String> = serde_json::from_str(&content).unwrap_or_default();
        blobs
            .get(address)
            .cloned()
            .ok_or_else(|| NapError::VcsError(format!("mock provenance blob not found: {address}")))
    }

    fn log(
        &self,
        repo_path: &Path,
        _file: Option<&str>,
        _limit: usize,
    ) -> Result<Vec<CommitInfo>, NapError> {
        Ok(self.load_commits(repo_path))
    }

    fn create_branch(&self, _path: &Path, name: &str) -> Result<(), NapError> {
        self.branches.lock().unwrap().push(name.to_string());
        Ok(())
    }

    fn switch_branch(&self, _path: &Path, _name: &str) -> Result<(), NapError> {
        Ok(())
    }

    fn current_branch(&self, _path: &Path) -> Result<String, NapError> {
        Ok("main".to_string())
    }

    fn head_hash(&self, repo_path: &Path) -> Result<String, NapError> {
        let commits = self.load_commits(repo_path);
        commits
            .last()
            .map(|c| c.id.clone())
            .ok_or_else(|| NapError::VcsError("no commits".to_string()))
    }

    fn resolve_branch_head(&self, repo_path: &Path, _branch: &str) -> Result<String, NapError> {
        let commits = self.load_commits(repo_path);
        commits
            .last()
            .map(|c| c.id.clone())
            .ok_or_else(|| NapError::VcsError("no commits on branch".to_string()))
    }

    fn revert(&self, repo_path: &Path, commit_hash: &str) -> Result<String, NapError> {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        let hash = format!("{:064x}", n);
        let mut commits = self.load_commits(repo_path);
        let info = CommitInfo {
            id: hash.clone(),
            parent: Some(commit_hash.to_string()),
            author: "revert".to_string(),
            message: format!("revert {}", commit_hash),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        commits.push(info);
        self.save_commits(repo_path, &commits);
        Ok(hash)
    }

    fn list_branches(&self, _path: &Path) -> Result<Vec<String>, NapError> {
        Ok(self.branches.lock().unwrap().clone())
    }

    fn add_remote(&self, _path: &Path, _name: &str, _url: &str) -> Result<(), NapError> {
        Ok(())
    }

    fn remove_remote(&self, _path: &Path, _name: &str) -> Result<(), NapError> {
        Ok(())
    }

    fn list_remotes(&self, _path: &Path) -> Result<Vec<(String, String)>, NapError> {
        Ok(Vec::new())
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
