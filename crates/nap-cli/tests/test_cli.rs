use assert_cmd::Command;
use clap::CommandFactory;
use nap_cli::Cli;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Every public command and subcommand must remain invocable through the
/// compiled binary. Functional Lore-backed coverage lives in
/// `local_lore_suite`; this catches command-surface regressions such as a
/// removed subcommand, changed dispatch name, or broken Clap definition.
#[test]
fn test_all_public_commands_expose_help() {
    fn collect_paths(command: &clap::Command, parent: &[String], paths: &mut Vec<Vec<String>>) {
        for subcommand in command.get_subcommands().filter(|cmd| !cmd.is_hide_set()) {
            let mut path = parent.to_vec();
            path.push(subcommand.get_name().to_owned());
            paths.push(path.clone());
            collect_paths(subcommand, &path, paths);
        }
    }

    let root = Cli::command();
    let mut commands = vec![Vec::new()];
    collect_paths(&root, &[], &mut commands);

    for args in commands {
        let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
        cmd.args(args)
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"));
    }
}

/// Exercise commands that operate entirely on supplied files and the two v0
/// stubs. Lore-backed stateful commands are covered in local_lore_suite.
#[test]
fn test_local_file_command_workflow() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let base = temp.path().join("base.yaml");
    let current = temp.path().join("current.yaml");
    let proposed = temp.path().join("proposed.yaml");
    let asset = temp.path().join("asset.txt");
    fs::write(&base, "name: Base\ncount: 1\n").unwrap();
    fs::write(&current, "name: Current\ncount: 1\n").unwrap();
    fs::write(&proposed, "name: Base\ncount: 2\n").unwrap();
    fs::write(&asset, "Narrative Addressing Protocol").unwrap();

    let run = |args: &[&str]| {
        let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
        cmd.args(args).assert().success();
    };
    run(&["schema", "manifest", "--format", "json"]);
    run(&["diff", base.to_str().unwrap(), current.to_str().unwrap()]);
    run(&[
        "merge",
        base.to_str().unwrap(),
        current.to_str().unwrap(),
        proposed.to_str().unwrap(),
    ]);
    run(&["content-hash", asset.to_str().unwrap()]);
    run(&["sign", "test-repository/character/testhero"]);
    run(&["verify", "test-repository/character/testhero"]);

    // Commands with intentionally unavailable external prerequisites must
    // fail descriptively rather than panic or mutate a user environment.
    let mut install = Command::cargo_bin("nap").expect("Failed to find nap binary");
    install
        .args(["install", "not-a-supported-target"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown target"));
    let mut validate = Command::cargo_bin("nap").expect("Failed to find nap binary");
    validate
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Provide either a NAP URI"));
}

#[test]
fn test_nap_resolve_accepts_uri_with_nap_scheme() {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.arg("resolve")
        .arg("nap://test-repository/character/testhero")
        .arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("resolve"));
}

#[test]
fn test_nap_resolve_accepts_uri_without_nap_scheme() {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.arg("resolve")
        .arg("test-repository/character/testhero")
        .arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("resolve"));
}

#[test]
fn test_nap_query_accepts_uri_with_nap_scheme() {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.arg("query")
        .arg("nap://test-repository/character/testhero")
        .arg("name")
        .arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("query"));
}

#[test]
fn test_nap_query_accepts_uri_without_nap_scheme() {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.arg("query")
        .arg("test-repository/character/testhero")
        .arg("name")
        .arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("query"));
}

#[test]
fn test_nap_resolve_accepts_uri_with_leading_slash_without_scheme() {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.arg("resolve")
        .arg("/test-repository/character/testhero")
        .arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("resolve"));
}

#[test]
fn test_nap_resolve_help_shows_provenance_flags() {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.arg("resolve").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--provenance"))
        .stdout(predicate::str::contains("--include-blobs"));
}

#[test]
fn test_nap_add_repr_alias_still_resolves_to_add_command() {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.arg("add-repr").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Add a file representation"));
}

#[test]
fn test_nap_presign_help_lists_safe_connection_options() {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.arg("presign").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--ttl-seconds"))
        .stdout(predicate::str::contains("--http-url"))
        .stdout(predicate::str::contains("--token-env"));
}

#[test]
fn test_nap_presign_rejects_branch_and_commit_together() {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.args([
        "presign",
        "nap://test-repository/character/testhero",
        "reference_image",
        "--branch",
        "main",
        "--commit",
        "abc",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

/// The global read-source selector and the push destination name must remain
/// distinct so ordinary commands never panic while parsing.
#[test]
fn test_remote_source_and_push_remote_name_flags_coexist() {
    let mut source = Command::cargo_bin("nap").expect("Failed to find nap binary");
    source
        .args(["--remote", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List repositories"));

    let mut push = Command::cargo_bin("nap").expect("Failed to find nap binary");
    push.args(["push", "example", "--remote-name", "origin", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Remote name"));

    let mut conflict = Command::cargo_bin("nap").expect("Failed to find nap binary");
    conflict
        .args(["--remote", "--local", "schema", "manifest"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}
