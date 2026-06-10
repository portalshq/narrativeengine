//! NAP Commit — the history primitive.
//!
//! A commit records a point-in-time snapshot of a manifest, plus
//! patch metadata describing what changed. This is Option C from the
//! design requirements: **Snapshot + Patch Metadata**.
//!
//! - The VCS (Git) stores the full snapshot (tree).
//! - The commit object stores change descriptions (patches) for efficient
//!   audit/provenance without requiring full diff reconstruction.
//!
//! Commits are NOT stored inside the manifest. The manifest stores only
//! `head` — a pointer to the latest commit hash.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A NAP commit — snapshot + patch metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    /// SHA-256 of commit content (self-referential hash).
    pub id: String,

    /// Parent commit hash. `None` for the initial commit.
    pub parent: Option<String>,

    /// When this commit was created.
    pub timestamp: DateTime<Utc>,

    /// Author identifier (DID key, email, or key fingerprint).
    pub author: String,

    /// Ed25519 signature over the commit hash (optional for v0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    /// Human-readable commit message.
    pub message: String,

    /// SHA-256 of the resulting manifest after this commit.
    pub manifest_hash: String,

    /// What changed in this commit (patch metadata for audit).
    #[serde(default)]
    pub changes: Vec<Change>,
}

/// A single change within a commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Dot-notation path to the changed field.
    /// e.g., `"properties.homeworld"`, `"representations.reference_image.hash"`.
    pub path: String,

    /// The kind of change.
    pub operation: ChangeOp,

    /// Previous value hash (for verification).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_value: Option<String>,

    /// New value hash or literal (for small values).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_value: Option<String>,
}

/// The type of change operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeOp {
    /// Set or update a field.
    Set,
    /// Delete a field.
    Delete,
    /// Append to an array/list.
    Append,
    /// Remove from an array/list.
    Remove,
}

impl Commit {
    /// Create a new commit. The `id` is computed after construction
    /// by hashing the serialized content.
    pub fn new(
        parent: Option<String>,
        author: &str,
        message: &str,
        manifest_hash: &str,
        changes: Vec<Change>,
    ) -> Self {
        let mut commit = Self {
            id: String::new(), // Placeholder — computed below
            parent,
            timestamp: Utc::now(),
            author: author.to_string(),
            signature: None,
            message: message.to_string(),
            manifest_hash: manifest_hash.to_string(),
            changes,
        };
        commit.id = commit.compute_id();
        commit
    }

    /// Compute the content-addressed ID (SHA-256) of this commit.
    /// Hashes: parent + timestamp + author + message + manifest_hash + changes.
    fn compute_id(&self) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(self.parent.as_deref().unwrap_or("root").as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(self.author.as_bytes());
        hasher.update(self.message.as_bytes());
        hasher.update(self.manifest_hash.as_bytes());

        // Include change paths and ops for determinism
        for change in &self.changes {
            hasher.update(change.path.as_bytes());
            hasher.update(format!("{:?}", change.operation).as_bytes());
        }

        let digest = hasher.finalize();
        hex::encode(digest)
    }

    /// Re-compute and verify the commit ID matches the stored value.
    pub fn verify_id(&self) -> bool {
        self.id == self.compute_id()
    }
}

impl Change {
    /// Create a `Set` change.
    pub fn set(path: &str, old_value: Option<String>, new_value: String) -> Self {
        Self {
            path: path.to_string(),
            operation: ChangeOp::Set,
            old_value,
            new_value: Some(new_value),
        }
    }

    /// Create a `Delete` change.
    pub fn delete(path: &str, old_value: String) -> Self {
        Self {
            path: path.to_string(),
            operation: ChangeOp::Delete,
            old_value: Some(old_value),
            new_value: None,
        }
    }

    /// Create an `Append` change.
    pub fn append(path: &str, new_value: String) -> Self {
        Self {
            path: path.to_string(),
            operation: ChangeOp::Append,
            old_value: None,
            new_value: Some(new_value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_id_determinism() {
        // Two commits with same content should get the same ID
        // (except timestamp — so we can't test exact equality easily).
        // Instead, verify that id matches recompute.
        let commit = Commit::new(
            None,
            "test-author",
            "initial commit",
            "sha256:abc123",
            vec![Change::set("properties.name", None, "Luke".to_string())],
        );
        assert!(commit.verify_id());
    }

    #[test]
    fn test_commit_with_parent() {
        let parent_commit = Commit::new(
            None,
            "test-author",
            "initial commit",
            "sha256:abc123",
            vec![],
        );
        let child_commit = Commit::new(
            Some(parent_commit.id.clone()),
            "test-author",
            "update homeworld",
            "sha256:def456",
            vec![Change::set(
                "properties.homeworld",
                None,
                "nap://starwars/location/tatooine".to_string(),
            )],
        );
        assert_eq!(child_commit.parent, Some(parent_commit.id));
        assert!(child_commit.verify_id());
    }
}
