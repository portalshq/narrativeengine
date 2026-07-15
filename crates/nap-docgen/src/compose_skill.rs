use crate::model::{DocMeta, WorkspaceMeta};
use crate::templates;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn compose_skill(
    workspace_root: &Path,
    cargo_meta: &WorkspaceMeta,
    meta: &DocMeta,
) -> Result<String> {
    let template_path = workspace_root.join("SKILL.template.md");
    if !template_path.exists() {
        anyhow::bail!("SKILL.template.md not found at {}", template_path.display());
    }

    let content = fs::read_to_string(&template_path)?;

    let variables = crate::compose_readme::build_variables(workspace_root, cargo_meta, meta)?;

    templates::expand_template(&content, workspace_root, &variables)
}
