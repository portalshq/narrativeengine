mod cargo_meta;
mod clap_walk;
mod compose_readme;
mod compose_skill;
mod dot;
mod examples;
mod filesystem;
mod help_snapshots;
mod json_out;
mod markdown;
mod mermaid;
mod model;
mod templates;
mod util;
mod yaml_out;

use anyhow::{Context, Result};
use model::DocMeta;
use std::fs;

fn main() -> Result<()> {
    let workspace_root = util::find_workspace_root().context("failed to locate workspace root")?;

    eprintln!("nap-docgen: workspace root = {}", workspace_root.display());

    // 1. Extract command tree from Clap
    let commands = clap_walk::extract_all_commands();
    eprintln!("nap-docgen: extracted {} commands", commands.len());

    // 2. Extract global options
    let global_options = clap_walk::extract_global_options();

    // 3. Read workspace metadata from Cargo.toml
    let cargo_meta = cargo_meta::read_workspace_meta(&workspace_root)?;
    eprintln!("nap-docgen: workspace version = {}", cargo_meta.version);

    // 4. Get git SHA for version stamping
    let git_sha = util::get_git_sha();

    // 5. Build doc metadata
    let doc_meta = DocMeta {
        generator_version: env!("CARGO_PKG_VERSION").to_string(),
        crate_version: cargo_meta.version.clone(),
        git_sha,
        generator_name: "nap-docgen".to_string(),
    };

    // 6. Ensure output directories exist
    fs::create_dir_all(workspace_root.join("docs/generated/commands"))
        .context("failed to create docs/generated/commands")?;
    fs::create_dir_all(workspace_root.join("docs/generated/help"))
        .context("failed to create docs/generated/help")?;
    fs::create_dir_all(workspace_root.join("docs/examples"))
        .context("failed to create docs/examples")?;
    fs::create_dir_all(workspace_root.join("docs/authored"))
        .context("failed to create docs/authored")?;

    let mut files_written = 0u32;
    let mut files_skipped = 0u32;

    // 7. Generate individual command pages
    for cmd in &commands {
        let example = examples::load_example_snippet(&workspace_root, &cmd.name);
        let page = markdown::render_command_page(cmd, &doc_meta, example.as_deref());
        let path = workspace_root
            .join("docs/generated/commands")
            .join(format!("{}.md", cmd.name));
        if filesystem::write_if_changed(&path, &page)? {
            files_written += 1;
        } else {
            files_skipped += 1;
        }
    }

    // 8. Generate sub-subcommand pages (e.g., remote add, remote ls, remote rm)
    for cmd in &commands {
        for sub in &cmd.subcommands {
            let example = examples::load_example_snippet(&workspace_root, &sub.name);
            let page = markdown::render_command_page(sub, &doc_meta, example.as_deref());
            let filename = format!("{}-{}.md", cmd.name, sub.name);
            let path = workspace_root
                .join("docs/generated/commands")
                .join(&filename);
            if filesystem::write_if_changed(&path, &page)? {
                files_written += 1;
            } else {
                files_skipped += 1;
            }
        }
    }

    // 9. Generate command index
    let index = markdown::render_command_index(&commands, &doc_meta);
    let index_path = workspace_root.join("docs/generated/command-index.md");
    if filesystem::write_if_changed(&index_path, &index)? {
        files_written += 1;
    } else {
        files_skipped += 1;
    }

    // 10. Generate CLI summary
    let cli_summary =
        markdown::render_cli_summary(&commands, &cargo_meta, &global_options, &doc_meta);
    let cli_path = workspace_root.join("docs/generated/cli.md");
    if filesystem::write_if_changed(&cli_path, &cli_summary)? {
        files_written += 1;
    } else {
        files_skipped += 1;
    }

    // 11. Generate global options page
    let options_page = markdown::render_global_options(&global_options, &doc_meta);
    let options_path = workspace_root.join("docs/generated/options.md");
    if filesystem::write_if_changed(&options_path, &options_page)? {
        files_written += 1;
    } else {
        files_skipped += 1;
    }

    // 12. Generate environment variables page
    let env_page = markdown::render_environment_page(&commands, &doc_meta);
    let env_path = workspace_root.join("docs/generated/environment.md");
    if filesystem::write_if_changed(&env_path, &env_page)? {
        files_written += 1;
    } else {
        files_skipped += 1;
    }

    // 13. Generate machine-readable JSON
    let json = json_out::render_commands_json(&commands, &cargo_meta, &doc_meta);
    let json_path = workspace_root.join("docs/generated/commands.json");
    if filesystem::write_if_changed(&json_path, &json)? {
        files_written += 1;
    } else {
        files_skipped += 1;
    }

    // 14. Generate machine-readable YAML
    let yaml = yaml_out::render_commands_yaml(&commands, &cargo_meta, &doc_meta);
    let yaml_path = workspace_root.join("docs/generated/commands.yaml");
    if filesystem::write_if_changed(&yaml_path, &yaml)? {
        files_written += 1;
    } else {
        files_skipped += 1;
    }

    // 15. Generate DOT graph
    let dot_graph = dot::render_command_graph(&commands);
    let dot_path = workspace_root.join("docs/generated/commands.dot");
    if filesystem::write_if_changed(&dot_path, &dot_graph)? {
        files_written += 1;
    } else {
        files_skipped += 1;
    }

    // 16. Generate Mermaid graph
    let mermaid_graph = mermaid::render_command_graph(&commands);
    let mermaid_path = workspace_root.join("docs/generated/commands.mermaid");
    if filesystem::write_if_changed(&mermaid_path, &mermaid_graph)? {
        files_written += 1;
    } else {
        files_skipped += 1;
    }

    // 17. Generate help snapshots (best-effort, non-fatal)
    match help_snapshots::generate_all(&workspace_root, &commands) {
        Ok(()) => eprintln!("nap-docgen: help snapshots generated"),
        Err(e) => eprintln!("nap-docgen: help snapshots skipped: {e}"),
    }

    // 18. Compose README.md
    match compose_readme::compose_readme(&workspace_root, &cargo_meta, &doc_meta) {
        Ok(readme) => {
            let readme_path = workspace_root.join("README.md");
            if filesystem::write_if_changed(&readme_path, &readme)? {
                files_written += 1;
            } else {
                files_skipped += 1;
            }
        }
        Err(e) => {
            eprintln!("nap-docgen: README composition skipped: {e}");
        }
    }

    // 19. Compose SKILL.md (root)
    match compose_skill::compose_root_skill(&workspace_root, &cargo_meta, &doc_meta) {
        Ok(skill) => {
            let skill_path = workspace_root.join("SKILL.md");
            if filesystem::write_if_changed(&skill_path, &skill)? {
                files_written += 1;
            } else {
                files_skipped += 1;
            }
        }
        Err(e) => {
            eprintln!("nap-docgen: SKILL.md composition skipped: {e}");
        }
    }

    // 20. Compose per-skill SKILL.md files from skills/templates/
    match compose_skill::compose_all_skills(&workspace_root, &cargo_meta, &doc_meta) {
        Ok(skills) => {
            for (skill_name, content) in skills {
                let skill_dir = workspace_root.join("skills").join(&skill_name);
                fs::create_dir_all(&skill_dir)
                    .with_context(|| format!("failed to create skills/{skill_name}"))?;
                let skill_path = skill_dir.join("SKILL.md");
                if filesystem::write_if_changed(&skill_path, &content)? {
                    files_written += 1;
                } else {
                    files_skipped += 1;
                }
            }
        }
        Err(e) => {
            eprintln!("nap-docgen: per-skill composition skipped: {e}");
        }
    }

    // 21. Summary
    eprintln!("nap-docgen: done.");
    eprintln!(
        "  Files written: {}, skipped (unchanged): {}",
        files_written, files_skipped
    );

    Ok(())
}
