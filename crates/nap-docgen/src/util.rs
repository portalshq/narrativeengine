use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn find_workspace_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .context("nap-docgen must be inside crates/")?
        .parent()
        .context("nap-docgen must be inside crates/<name>/")?;
    Ok(workspace_root.to_path_buf())
}

pub fn escape_markdown(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('_', "\\_")
        .replace('*', "\\*")
        .replace('`', "\\`")
        .replace('\n', " ")
}

pub fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n")
}

pub fn ensure_trailing_newline(text: &str) -> String {
    if text.ends_with('\n') {
        text.to_string()
    } else {
        format!("{text}\n")
    }
}
