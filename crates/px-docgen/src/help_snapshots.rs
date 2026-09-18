use crate::model::CommandModel;
use crate::util;
use anyhow::Result;
use std::collections::BTreeSet;
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

    remove_stale_help_snapshots(&help_dir, &expected_snapshot_names(commands))?;

    Ok(())
}

fn expected_snapshot_names(commands: &[CommandModel]) -> BTreeSet<String> {
    let mut names = BTreeSet::from(["px.txt".to_string()]);
    for command in commands {
        collect_snapshot_names(command, &mut names);
    }
    names
}

fn collect_snapshot_names(command: &CommandModel, names: &mut BTreeSet<String>) {
    names.insert(format!("px--{}.txt", command.full_path.replace(' ', "--")));
    for subcommand in &command.subcommands {
        collect_snapshot_names(subcommand, names);
    }
}

fn remove_stale_help_snapshots(help_dir: &Path, expected: &BTreeSet<String>) -> Result<()> {
    for entry in fs::read_dir(help_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let filename = entry.file_name().to_string_lossy().to_string();
        if filename.starts_with("px") && filename.ends_with(".txt") && !expected.contains(&filename)
        {
            fs::remove_file(entry.path())?;
        }
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
    // px and px-docgen are siblings in the same directory
    let exe_dir = std::env::current_exe()?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("cannot determine binary directory"))?
        .to_path_buf();

    let binary = exe_dir.join("px");

    if !binary.exists() {
        anyhow::bail!("px binary not found at {}", binary.display());
    }

    let mut cmd_args: Vec<&str> = subcommand_args.to_vec();
    cmd_args.push("--help");

    let output = Command::new(&binary).args(&cmd_args).output()?;

    if output.status.success() {
        let filename = if subcommand_args.is_empty() {
            "px.txt".to_string()
        } else {
            let name = subcommand_args.join("--");
            format!("px--{name}.txt")
        };

        let path = help_dir.join(&filename);
        let help = String::from_utf8_lossy(&output.stdout);
        let help = help
            .lines()
            .map(|line| if line.trim().is_empty() { "" } else { line })
            .collect::<Vec<_>>()
            .join("\n");
        let content = util::ensure_trailing_newline(&help);
        crate::filesystem::atomic_write(&path, &content)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_help_snapshots_are_removed() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("px.txt"), "root").unwrap();
        fs::write(tmp.path().join("px--push.txt"), "push").unwrap();
        fs::write(tmp.path().join("px--publish.txt"), "publish").unwrap();
        fs::write(tmp.path().join("notes.txt"), "keep").unwrap();

        let expected = BTreeSet::from(["px.txt".to_string(), "px--push.txt".to_string()]);
        remove_stale_help_snapshots(tmp.path(), &expected).unwrap();

        assert!(tmp.path().join("px.txt").exists());
        assert!(tmp.path().join("px--push.txt").exists());
        assert!(!tmp.path().join("px--publish.txt").exists());
        assert!(tmp.path().join("notes.txt").exists());
    }
}
