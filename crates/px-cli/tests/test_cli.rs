use assert_cmd::Command;
use clap::CommandFactory;
use predicates::prelude::*;
use px_cli::Cli;
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
        let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
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
    fs::write(&asset, "PX protocol").unwrap();

    let run = |args: &[&str]| {
        let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
        cmd.args(args).assert().success();
    };
    run(&["schema", "manifest", "--format", "json"]);
    let mut legacy_diff = Command::cargo_bin("px").expect("Failed to find px binary");
    legacy_diff
        .args(["diff", base.to_str().unwrap(), current.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
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
    let mut install = Command::cargo_bin("px").expect("Failed to find px binary");
    install
        .args(["install", "not-a-supported-target"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown target"));
    let mut validate = Command::cargo_bin("px").expect("Failed to find px binary");
    validate
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Provide either a PX URI"));
}

#[test]
fn test_px_resolve_accepts_uri_with_px_scheme() {
    let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
    cmd.arg("resolve")
        .arg("px://test-repository/character/testhero")
        .arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("resolve"));
}

#[test]
fn test_px_resolve_rejects_local_with_revision_selectors() {
    let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
    cmd.args([
        "--local",
        "resolve",
        "test-repository/character/testhero",
        "--branch",
        "main",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "use --local or --branch/--commit, not both",
    ));
}

#[test]
fn test_px_resolve_accepts_uri_without_px_scheme() {
    let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
    cmd.arg("resolve")
        .arg("test-repository/character/testhero")
        .arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("resolve"));
}

#[test]
fn test_px_query_accepts_uri_with_px_scheme() {
    let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
    cmd.arg("query")
        .arg("px://test-repository/character/testhero")
        .arg("name")
        .arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("query"));
}

#[test]
fn test_px_query_accepts_uri_without_px_scheme() {
    let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
    cmd.arg("query")
        .arg("test-repository/character/testhero")
        .arg("name")
        .arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("query"));
}

#[test]
fn test_px_resolve_accepts_uri_with_leading_slash_without_scheme() {
    let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
    cmd.arg("resolve")
        .arg("/test-repository/character/testhero")
        .arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("resolve"));
}

#[test]
fn test_px_resolve_help_shows_provenance_flags() {
    let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
    cmd.arg("resolve").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--provenance"))
        .stdout(predicate::str::contains("--include-blobs"));
}

#[test]
fn test_px_add_repr_alias_still_resolves_to_add_command() {
    let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
    cmd.arg("add-repr").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Add a file representation"));
}

#[test]
fn test_px_presign_help_lists_safe_connection_options() {
    let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
    cmd.arg("presign").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--ttl-seconds"))
        .stdout(predicate::str::contains("--http-url"))
        .stdout(predicate::str::contains("--token-env"));
}

#[test]
fn test_px_presign_rejects_branch_and_commit_together() {
    let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
    cmd.args([
        "presign",
        "px://test-repository/character/testhero",
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
    let mut source = Command::cargo_bin("px").expect("Failed to find px binary");
    source
        .args(["--remote", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List repositories"));

    let mut push = Command::cargo_bin("px").expect("Failed to find px binary");
    push.args(["push", "example", "--remote-name", "origin", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Remote name"));

    let mut conflict = Command::cargo_bin("px").expect("Failed to find px binary");
    conflict
        .args(["--remote", "--local", "schema", "manifest"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_configure_unified_command() {
    // Bare `px configure --help` should succeed and mention providers.
    let mut help = Command::cargo_bin("px").expect("Failed to find px binary");
    help.args(["configure", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Configure version-control backend",
        ))
        .stdout(predicate::str::contains("local"))
        .stdout(predicate::str::contains("remote"));

    // `px configure status --help` should also succeed.
    let mut status_help = Command::cargo_bin("px").expect("Failed to find px binary");
    status_help
        .args(["configure", "status", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));

    // Deprecated aliases must still be invocable (hidden but functional).
    let mut choose_help = Command::cargo_bin("px").expect("Failed to find px binary");
    choose_help
        .args(["choose", "backend", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Provider type"));

    let mut backend_help = Command::cargo_bin("px").expect("Failed to find px binary");
    backend_help
        .args(["backend", "configure", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Backend type"));

    for args in [
        vec!["config", "--help"],
        vec!["publish", "example", "--help"],
        vec!["head-hash", "example", "--help"],
        vec!["head_hash", "example", "--help"],
    ] {
        let mut alias = Command::cargo_bin("px").expect("Failed to find px binary");
        alias.args(args).assert().success();
    }
}

#[test]
fn test_configure_sets_provider_local_and_remote() {
    // `px configure local` should create a provider.toml with local type.
    let tmp = TempDir::new().unwrap();
    let mut cfg = Command::cargo_bin("px").expect("Failed to find px binary");
    cfg.args([
        "configure",
        "local",
        "--base-dir",
        tmp.path().to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Configured local"));

    let provider_toml = std::fs::read_to_string(tmp.path().join("provider.toml")).unwrap();
    assert!(provider_toml.contains("provider_type = \"local\""));

    // `px configure remote --remote-url` should work and persist.
    let tmp2 = TempDir::new().unwrap();
    let mut cfg2 = Command::cargo_bin("px").expect("Failed to find px binary");
    cfg2.args([
        "configure",
        "remote",
        "--remote-url",
        "lore://192.168.0.27:41337",
        "--base-dir",
        tmp2.path().to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Configured remote"));

    let provider_toml2 = std::fs::read_to_string(tmp2.path().join("provider.toml")).unwrap();
    assert!(provider_toml2.contains("remote"));
    assert!(provider_toml2.contains("192.168.0.27"));

    // Alias --endpoint should also work.
    let tmp3 = TempDir::new().unwrap();
    let mut cfg3 = Command::cargo_bin("px").expect("Failed to find px binary");
    cfg3.args([
        "configure",
        "remote",
        "--endpoint",
        "lore://10.0.0.1:41337",
        "--base-dir",
        tmp3.path().to_str().unwrap(),
    ])
    .assert()
    .success();
    let provider_toml3 = std::fs::read_to_string(tmp3.path().join("provider.toml")).unwrap();
    assert!(provider_toml3.contains("10.0.0.1"));
}

#[test]
fn test_configure_validates_before_resetting_existing_provider() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("provider.toml");
    let original = "provider_type = \"remote\"\nremote_url = \"lore://old.example:41337\"\nworkspace_id = \"default\"\n";
    fs::write(&config_path, original).unwrap();

    let mut missing_provider = Command::cargo_bin("px").expect("Failed to find px binary");
    missing_provider
        .args([
            "configure",
            "--reset",
            "--base-dir",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("provider type required"));
    assert_eq!(fs::read_to_string(&config_path).unwrap(), original);

    let mut conflicting_providers = Command::cargo_bin("px").expect("Failed to find px binary");
    conflicting_providers
        .args([
            "configure",
            "local",
            "--provider",
            "remote",
            "--base-dir",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
    assert_eq!(fs::read_to_string(&config_path).unwrap(), original);
}

#[test]
fn test_completions_command_is_removed() {
    let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
    cmd.args(["completions", "bash"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}
