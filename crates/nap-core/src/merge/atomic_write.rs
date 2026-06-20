//! Atomic file write utility.
//!
//! Protocol invariant: repository writes MUST be atomic.
//!
//! Implementation: write temp file → flush → fsync → rename.
//! This prevents partial writes from corrupting the repository.

use std::io::Write;
use std::path::{Path, PathBuf};

/// Error type for atomic write operations.
#[derive(Debug, thiserror::Error)]
pub enum AtomicWriteError {
    #[error("failed to create temp file: {0}")]
    CreateTemp(std::io::Error),

    #[error("failed to write content: {0}")]
    Write(std::io::Error),

    #[error("failed to flush: {0}")]
    Flush(std::io::Error),

    #[error("failed to fsync: {0}")]
    Fsync(std::io::Error),

    #[error("failed to rename: {0}")]
    Rename(std::io::Error),
}

/// Atomically write content to a file.
///
/// 1. Create a temporary file alongside the target path.
/// 2. Write all content.
/// 3. Flush the stream.
/// 4. fsync the file (ensures data hits the disk).
/// 5. Rename the temp file to the target path (atomic on Unix).
///
/// # Arguments
///
/// * `path` - The final file path to write.
/// * `content` - The bytes to write.
pub fn atomic_write(path: &Path, content: &[u8]) -> Result<(), AtomicWriteError> {
    // Create temp file in the same directory (same filesystem = atomic rename)
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("nap_tmp");
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("tmp");

    let temp_path = loop {
        let rand_suffix: u64 = rand::random();
        let candidate = parent.join(format!(".{file_stem}_{rand_suffix:016x}.{extension}"));
        if !candidate.exists() {
            break candidate;
        }
    };

    let result = try_atomic_write(path, &temp_path, content);

    // Clean up temp file on error
    if result.is_err() {
        let _ = std::fs::remove_file(&temp_path);
    }

    result
}

fn try_atomic_write(
    target: &Path,
    temp_path: &PathBuf,
    content: &[u8],
) -> Result<(), AtomicWriteError> {
    // Ensure parent directory exists
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(AtomicWriteError::CreateTemp)?;
    }

    // Create and write to temp file
    let mut file = std::fs::File::create(temp_path).map_err(AtomicWriteError::CreateTemp)?;
    file.write_all(content).map_err(AtomicWriteError::Write)?;
    file.flush().map_err(AtomicWriteError::Flush)?;
    file.sync_all().map_err(AtomicWriteError::Fsync)?;
    drop(file); // Close before rename

    // Atomic rename (atomic on Unix, best-effort on Windows)
    std::fs::rename(temp_path, target).map_err(AtomicWriteError::Rename)?;

    // Sync the directory to ensure the rename is durable
    if let Some(parent) = target.parent()
        && let Ok(dir) = std::fs::File::open(parent)
    {
        let _ = dir.sync_all();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_atomic_write_and_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.yaml");

        let content = b"key: value\nname: Luke\n";
        atomic_write(&path, content).unwrap();

        let mut file = std::fs::File::open(&path).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, content);
    }

    #[test]
    fn test_atomic_write_no_temp_file_left() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("output.yaml");

        atomic_write(&path, b"hello").unwrap();

        // Verify no temp files remain
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name())
            .collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], "output.yaml");
    }

    #[test]
    fn test_atomic_write_content_is_identical() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest.yaml");

        let content = b"id: nap://test/char/luke\nname: Luke Skywalker\nversion: 1\n";
        atomic_write(&path, content).unwrap();

        let read_back = std::fs::read(&path).unwrap();
        assert_eq!(read_back, content);
    }

    #[test]
    fn test_atomic_write_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/sub/dir/manifest.yaml");

        atomic_write(&path, b"hello").unwrap();
        assert!(path.exists());
    }
}
