use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;

pub fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let dir = path.parent().context("no parent directory")?;
    fs::create_dir_all(dir)?;

    let mut tmp = NamedTempFile::new_in(dir)?;
    std::io::Write::write_all(&mut tmp, content.as_bytes())?;
    tmp.persist(path).context("failed to persist temp file")?;
    Ok(())
}

pub fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    if path.exists() {
        let existing = fs::read_to_string(path).context("failed to read existing file")?;
        if existing == content {
            return Ok(false);
        }
    }
    atomic_write(path, content)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_atomic_write_creates_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("subdir").join("test.md");
        atomic_write(&path, "hello").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
    }

    #[test]
    fn test_write_if_changed_skips_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.md");
        fs::write(&path, "content").unwrap();
        let written = write_if_changed(&path, "content").unwrap();
        assert!(!written);
    }

    #[test]
    fn test_write_if_changed_writes_different() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.md");
        fs::write(&path, "old").unwrap();
        let written = write_if_changed(&path, "new").unwrap();
        assert!(written);
        assert_eq!(fs::read_to_string(&path).unwrap(), "new");
    }
}
