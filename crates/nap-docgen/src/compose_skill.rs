use crate::model::{DocMeta, WorkspaceMeta};
use crate::templates;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Compose the root SKILL.md from SKILL.template.md (legacy).
pub fn compose_root_skill(
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

/// Discover all skill templates in skills/templates/*.md and compose
/// each into skills/<skill-name>/SKILL.md.
pub fn compose_all_skills(
    workspace_root: &Path,
    cargo_meta: &WorkspaceMeta,
    meta: &DocMeta,
) -> Result<Vec<(String, String)>> {
    let templates_dir = workspace_root.join("skills").join("templates");
    if !templates_dir.exists() {
        anyhow::bail!("skills/templates/ not found at {}", templates_dir.display());
    }

    let variables = crate::compose_readme::build_variables(workspace_root, cargo_meta, meta)?;

    let mut results = Vec::new();

    let entries: Vec<PathBuf> = fs::read_dir(&templates_dir)
        .context("failed to read skills/templates/")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.path())
        .collect();

    let mut entries: Vec<_> = entries
        .iter()
        .filter_map(|p| {
            p.file_stem()
                .map(|s| (s.to_string_lossy().to_string(), p.clone()))
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (skill_name, template_path) in &entries {
        let content = fs::read_to_string(template_path)
            .with_context(|| format!("failed to read {}", template_path.display()))?;

        let composed = templates::expand_template(&content, workspace_root, &variables)
            .with_context(|| format!("failed to expand template for skill '{skill_name}'"))?;

        results.push((skill_name.clone(), composed));
    }

    Ok(results)
}
