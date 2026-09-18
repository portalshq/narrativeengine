use anyhow::{Context, Result};
use std::collections::BTreeSet;
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

/// Remove only generated command pages that no longer correspond to the
/// current public Clap command tree. This keeps renamed or hidden commands
/// from lingering in the published reference while leaving other files alone.
pub fn remove_stale_command_pages(directory: &Path, expected: &BTreeSet<String>) -> Result<u32> {
    let mut removed = 0;
    for entry in fs::read_dir(directory).with_context(|| {
        format!(
            "failed to read generated command directory {}",
            directory.display()
        )
    })? {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("md")
        {
            continue;
        }
        let filename = entry.file_name().to_string_lossy().to_string();
        if !expected.contains(&filename) {
            fs::remove_file(&path).with_context(|| {
                format!(
                    "failed to remove stale generated command page {}",
                    path.display()
                )
            })?;
            removed += 1;
        }
    }
    Ok(removed)
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

    #[test]
    fn test_remove_stale_command_pages_only_removes_unexpected_markdown() {
        let tmp = tempfile::tempdir().unwrap();
        let commands = tmp.path().join("commands");
        fs::create_dir_all(&commands).unwrap();
        fs::write(commands.join("keep.md"), "keep").unwrap();
        fs::write(commands.join("stale.md"), "stale").unwrap();
        fs::write(commands.join("data.json"), "{}").unwrap();

        let expected = BTreeSet::from(["keep.md".to_string()]);
        assert_eq!(remove_stale_command_pages(&commands, &expected).unwrap(), 1);
        assert!(commands.join("keep.md").exists());
        assert!(!commands.join("stale.md").exists());
        assert!(commands.join("data.json").exists());
    }
}
