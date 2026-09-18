use anyhow::{Context, Result};
use clap::CommandFactory;
use clap_complete::Shell;
use px_cli::Cli;
use std::path::Path;

pub fn generate_all(workspace_root: &Path) -> Result<u32> {
    let completion_dir = workspace_root.join("completions");
    let mut files_written = 0;

    for (shell, filename) in [
        (Shell::Bash, "px.bash"),
        (Shell::Zsh, "px.zsh"),
        (Shell::Fish, "px.fish"),
    ] {
        let content = render(shell)?;
        if crate::filesystem::write_if_changed(&completion_dir.join(filename), &content)? {
            files_written += 1;
        }
    }

    Ok(files_written)
}

fn render(shell: Shell) -> Result<String> {
    let mut command = Cli::command();
    let mut output = Vec::new();
    clap_complete::generate(shell, &mut command, "px", &mut output);
    String::from_utf8(output).context("completion generator returned non-UTF-8 output")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_completion_scripts_for_supported_shells() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
            let content = render(shell).unwrap();
            assert!(content.contains("px"));
            assert!(content.contains("configure"));
        }
    }
}
