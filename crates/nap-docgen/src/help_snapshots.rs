use crate::model::CommandModel;
use crate::util;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn generate_all(workspace_root: &Path, commands: &[CommandModel]) -> Result<()> {
    let help_dir = workspace_root.join("docs").join("generated").join("help");
    fs::create_dir_all(&help_dir)?;

    let _ = generate_help_snapshot(&help_dir, &[]);

    for cmd in commands {
        let args: Vec<&str> = cmd.full_path.split(' ').collect();
        let _ = generate_help_snapshot(&help_dir, &args);
        generate_subcommand_snapshots(&help_dir, cmd, &cmd.full_path);
    }

    Ok(())
}

fn generate_subcommand_snapshots(help_dir: &Path, cmd: &CommandModel, parent_path: &str) {
    for sub in &cmd.subcommands {
        let full = format!("{parent_path} {}", sub.name);
        let args: Vec<&str> = full.split(' ').collect();
        let _ = generate_help_snapshot(help_dir, &args);
        generate_subcommand_snapshots(help_dir, sub, &full);
    }
}

fn generate_help_snapshot(help_dir: &Path, subcommand_args: &[&str]) -> Result<()> {
    // nap and nap-docgen are siblings in the same directory
    let exe_dir = std::env::current_exe()?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("cannot determine binary directory"))?
        .to_path_buf();

    let binary = exe_dir.join("nap");

    if !binary.exists() {
        anyhow::bail!("nap binary not found at {}", binary.display());
    }

    let mut cmd_args: Vec<&str> = subcommand_args.to_vec();
    cmd_args.push("--help");

    let output = Command::new(&binary).args(&cmd_args).output()?;

    if output.status.success() {
        let filename = if subcommand_args.is_empty() {
            "nap.txt".to_string()
        } else {
            let name = subcommand_args.join("--");
            format!("nap--{name}.txt")
        };

        let path = help_dir.join(&filename);
        let content = util::ensure_trailing_newline(&String::from_utf8_lossy(&output.stdout));
        crate::filesystem::atomic_write(&path, &content)?;
    }

    Ok(())
}
