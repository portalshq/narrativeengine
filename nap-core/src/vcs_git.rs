//! Git backend implementation via shell commands.
//!
//! For v0, we use git CLI commands via `std::process::Command` for reliability
//! and simplicity. The `gix` crate is excellent but its API is large and
//! evolving — shelling out to git gives us battle-tested behavior with
//! minimal code for the prototype.
//!
//! Every git operation includes verbose tracing for debug visibility.

use std::path::Path;
use std::process::Command;

use tracing::{debug, trace};

use crate::error::NapError;
use crate::vcs::{CommitInfo, VcsBackend};

/// Git-backed VCS implementation.
#[derive(Debug, Default)]
pub struct GitBackend;

impl GitBackend {
    pub fn new() -> Self {
        Self
    }

    /// Run a git command and return stdout. Logs the command and output at trace level.
    fn run_git(path: &Path, args: &[&str]) -> Result<String, NapError> {
        let args_display = args.join(" ");
        trace!(
            cwd = %path.display(),
            command = %format!("git {args_display}"),
            "executing git command"
        );

        let output = Command::new("git")
            .args(args)
            .current_dir(path)
            .output()
            .map_err(|e| NapError::VcsError(format!("failed to execute git {args_display}: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            debug!(
                command = %format!("git {args_display}"),
                stderr = %stderr,
                exit_code = ?output.status.code(),
                "git command failed"
            );
            return Err(NapError::VcsError(format!(
                "git {args_display} failed: {stderr}"
            )));
        }

        trace!(
            command = %format!("git {args_display}"),
            stdout_len = stdout.len(),
            "git command succeeded"
        );
        Ok(stdout.trim().to_string())
    }
}

impl VcsBackend for GitBackend {
    fn init(&self, path: &Path) -> Result<(), NapError> {
        debug!(path = %path.display(), "initializing git repository");
        std::fs::create_dir_all(path)?;
        Self::run_git(path, &["init"])?;
        // Set default branch to "main"
        Self::run_git(path, &["checkout", "-b", "main"]).ok();
        // Configure user for commits (local to this repo)
        Self::run_git(path, &["config", "user.email", "nap@cinematiccanvas.com"])?;
        Self::run_git(path, &["config", "user.name", "NAP"])?;
        debug!(path = %path.display(), "git repository initialized");
        Ok(())
    }

    fn commit(&self, path: &Path, message: &str, author: &str) -> Result<String, NapError> {
        debug!(
            path = %path.display(),
            message = %message,
            author = %author,
            "creating git commit"
        );

        // Stage all changes
        Self::run_git(path, &["add", "-A"])?;

        // Check if there are staged changes
        let status = Self::run_git(path, &["status", "--porcelain"])?;
        if status.is_empty() {
            return Err(NapError::VcsError("nothing to commit".to_string()));
        }

        // Commit with author override
        let author_str = format!("{author} <{author}@nap>");
        Self::run_git(path, &["commit", "-m", message, "--author", &author_str])?;

        // Return the commit hash
        let hash = Self::run_git(path, &["rev-parse", "HEAD"])?;
        debug!(commit_hash = %hash, "git commit created");
        Ok(hash)
    }

    fn read_file_at_ref(
        &self,
        repo_path: &Path,
        file_path: &str,
        reference: Option<&str>,
    ) -> Result<String, NapError> {
        match reference {
            Some(git_ref) => {
                trace!(
                    repo = %repo_path.display(),
                    file = %file_path,
                    git_ref = %git_ref,
                    "reading file at ref"
                );
                let spec = format!("{git_ref}:{file_path}");
                Self::run_git(repo_path, &["show", &spec])
            }
            None => {
                trace!(
                    repo = %repo_path.display(),
                    file = %file_path,
                    "reading file from working tree"
                );
                let full_path = repo_path.join(file_path);
                std::fs::read_to_string(&full_path).map_err(|e| NapError::ManifestNotFound(
                    format!("{}: {e}", full_path.display())
                ))
            }
        }
    }

    fn log(&self, path: &Path, file: Option<&str>, limit: usize) -> Result<Vec<CommitInfo>, NapError> {
        let limit_str = format!("-{limit}");
        let format_flag = format!("--format=%H%n%P%n%an%n%s%n%aI%n---");

        let mut args = vec!["log", &limit_str, &format_flag];
        if let Some(file_path) = file {
            args.push("--");
            args.push(file_path);
        }

        let output = Self::run_git(path, &args)?;
        if output.is_empty() {
            return Ok(vec![]);
        }

        let mut commits = Vec::new();
        for entry in output.split("---\n") {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let lines: Vec<&str> = entry.lines().collect();
            if lines.len() >= 5 {
                commits.push(CommitInfo {
                    id: lines[0].to_string(),
                    parent: if lines[1].is_empty() {
                        None
                    } else {
                        Some(lines[1].split_whitespace().next().unwrap_or("").to_string())
                    },
                    author: lines[2].to_string(),
                    message: lines[3].to_string(),
                    timestamp: lines[4].to_string(),
                });
            }
        }

        Ok(commits)
    }

    fn create_branch(&self, path: &Path, name: &str) -> Result<(), NapError> {
        debug!(path = %path.display(), branch = %name, "creating git branch");
        Self::run_git(path, &["branch", name])?;
        Ok(())
    }

    fn switch_branch(&self, path: &Path, name: &str) -> Result<(), NapError> {
        debug!(path = %path.display(), branch = %name, "switching git branch");
        Self::run_git(path, &["checkout", name])?;
        Ok(())
    }

    fn create_tag(&self, path: &Path, name: &str) -> Result<(), NapError> {
        debug!(path = %path.display(), tag = %name, "creating git tag");
        Self::run_git(path, &["tag", name])?;
        Ok(())
    }

    fn current_branch(&self, path: &Path) -> Result<String, NapError> {
        Self::run_git(path, &["rev-parse", "--abbrev-ref", "HEAD"])
    }

    fn head_hash(&self, path: &Path) -> Result<String, NapError> {
        Self::run_git(path, &["rev-parse", "HEAD"])
    }

    fn list_branches(&self, path: &Path) -> Result<Vec<String>, NapError> {
        let output = Self::run_git(path, &["branch", "--format=%(refname:short)"])?;
        Ok(output.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
    }

    fn list_tags(&self, path: &Path) -> Result<Vec<String>, NapError> {
        let output = Self::run_git(path, &["tag", "--list"])?;
        Ok(output.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
    }

    fn revert(&self, path: &Path, commit_hash: &str) -> Result<String, NapError> {
        debug!(
            path = %path.display(),
            commit = %commit_hash,
            "reverting git commit"
        );

        // Git revert requires a clean working tree.  NAP's design leaves
        // head-pointer updates uncommitted.  Stash them, do the revert,
        // then drop the stash — the caller (Repository::revert_commit)
        // regenerates fresh head pointers afterward.
        let status = Self::run_git(path, &["status", "--porcelain"])?;
        let had_dirty = !status.is_empty();
        if had_dirty {
            Self::run_git(path, &["stash", "push", "-m", "nap-revert-stash"])?;
        }

        Self::run_git(path, &["revert", "--no-edit", commit_hash])?;

        // Discard the stash — the caller will regenerate head pointers
        if had_dirty {
            Self::run_git(path, &["stash", "drop"]).ok();
        }

        let hash = Self::run_git(path, &["rev-parse", "HEAD"])?;
        debug!(revert_commit = %hash, "git revert created");
        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_git_init_and_commit() {
        let tmp = TempDir::new().unwrap();
        let backend = GitBackend::new();

        // Init repo
        backend.init(tmp.path()).unwrap();

        // Create a file
        std::fs::write(tmp.path().join("test.txt"), "hello").unwrap();

        // Commit
        let hash = backend.commit(tmp.path(), "initial commit", "test-user").unwrap();
        assert!(!hash.is_empty());

        // Verify HEAD
        let head = backend.head_hash(tmp.path()).unwrap();
        assert_eq!(hash, head);
    }

    #[test]
    fn test_git_read_file_at_ref() {
        let tmp = TempDir::new().unwrap();
        let backend = GitBackend::new();
        backend.init(tmp.path()).unwrap();

        // Commit v1
        std::fs::write(tmp.path().join("data.txt"), "version 1").unwrap();
        let hash_v1 = backend.commit(tmp.path(), "v1", "user").unwrap();

        // Commit v2
        std::fs::write(tmp.path().join("data.txt"), "version 2").unwrap();
        backend.commit(tmp.path(), "v2", "user").unwrap();

        // Read current (v2)
        let current = backend.read_file_at_ref(tmp.path(), "data.txt", None).unwrap();
        assert_eq!(current, "version 2");

        // Read at v1
        let at_v1 = backend.read_file_at_ref(tmp.path(), "data.txt", Some(&hash_v1)).unwrap();
        assert_eq!(at_v1, "version 1");
    }

    #[test]
    fn test_git_log() {
        let tmp = TempDir::new().unwrap();
        let backend = GitBackend::new();
        backend.init(tmp.path()).unwrap();

        std::fs::write(tmp.path().join("a.txt"), "a").unwrap();
        backend.commit(tmp.path(), "first", "user").unwrap();

        std::fs::write(tmp.path().join("b.txt"), "b").unwrap();
        backend.commit(tmp.path(), "second", "user").unwrap();

        let log = backend.log(tmp.path(), None, 10).unwrap();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].message, "second");
        assert_eq!(log[1].message, "first");
    }

    #[test]
    fn test_git_branches() {
        let tmp = TempDir::new().unwrap();
        let backend = GitBackend::new();
        backend.init(tmp.path()).unwrap();

        std::fs::write(tmp.path().join("init.txt"), "init").unwrap();
        backend.commit(tmp.path(), "init", "user").unwrap();

        backend.create_branch(tmp.path(), "canon").unwrap();
        let branches = backend.list_branches(tmp.path()).unwrap();
        assert!(branches.contains(&"main".to_string()));
        assert!(branches.contains(&"canon".to_string()));
    }

    #[test]
    fn test_git_tags() {
        let tmp = TempDir::new().unwrap();
        let backend = GitBackend::new();
        backend.init(tmp.path()).unwrap();

        std::fs::write(tmp.path().join("init.txt"), "init").unwrap();
        backend.commit(tmp.path(), "init", "user").unwrap();

        backend.create_tag(tmp.path(), "v1.0").unwrap();
        let tags = backend.list_tags(tmp.path()).unwrap();
        assert!(tags.contains(&"v1.0".to_string()));
    }
}
