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
