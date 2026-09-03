use assert_cmd::Command;
use predicates::prelude::*;

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
