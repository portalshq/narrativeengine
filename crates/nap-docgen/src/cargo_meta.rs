use crate::model::WorkspaceMeta;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(serde::Deserialize)]
struct CargoToml {
    workspace: Option<WorkspaceSection>,
}

#[derive(serde::Deserialize)]
struct WorkspaceSection {
    package: Option<PackageSection>,
}

#[derive(serde::Deserialize, Default)]
struct PackageSection {
    version: Option<String>,
    license: Option<String>,
    authors: Option<Vec<String>>,
}

fn git_remote_url(workspace_root: &Path) -> Option<String> {
    Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(workspace_root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn read_workspace_meta(workspace_root: &Path) -> Result<WorkspaceMeta> {
    let content = fs::read_to_string(workspace_root.join("Cargo.toml"))
        .context("failed to read workspace Cargo.toml")?;
    let toml: CargoToml =
        toml::from_str(&content).context("failed to parse workspace Cargo.toml")?;

    let pkg = toml.workspace.and_then(|w| w.package).unwrap_or_default();

    let repository = git_remote_url(workspace_root);

    Ok(WorkspaceMeta {
        name: "nap".to_string(),
        version: pkg.version.unwrap_or_else(|| "0.0.0".to_string()),
        license: pkg.license.unwrap_or_else(|| "MIT".to_string()),
        authors: pkg.authors.unwrap_or_default(),
        repository,
        homepage: None,
        description: "Narrative Addressing Protocol".to_string(),
    })
}
