//! Integration Test Suite: Documentation Workflows
//!
//! This suite tests nap functionality against a local lore server,
//! following the "Quick Start" and "Usage Guide" workflows from documentation.
//!
//! Prerequisites:
//!   - A running local lore server at `lore://localhost:41337` (for provider validation)
//!     and `grpc://localhost:41337` (for actual CLI operations).
//!   - The `lore` binary in PATH.
//!
//! Known Issues:
//!   - QUIC transport (`lore://`) currently hangs on macOS in this environment.
//!     `grpc://` is used for test CLI operations as a reliable transport.
//!
//! Run with:
//!   cargo test -p nap-cli --test workflow_integration_suite --features lore-e2e -- --test-threads=1

#[cfg(feature = "workflow-e2e")]
use assert_cmd::Command;
#[cfg(feature = "workflow-e2e")]
use predicates::prelude::*;
#[cfg(feature = "workflow-e2e")]
use std::path::Path;
#[cfg(feature = "workflow-e2e")]
use tempfile::TempDir;

#[cfg(feature = "workflow-e2e")]
/// Helper to get the nap binary command, correctly configured for testing.
///
/// Uses `--provider remote` to avoid each test starting its own lore server
/// daemon (which would conflict on ports 41337/41339).  The lore server must
/// already be running at `lore://localhost:41337`.
fn nap_cmd(nap_home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.timeout(std::time::Duration::from_secs(30));

    // Set NAP_DIR to the temporary home directory to isolate the config
    cmd.env("NAP_DIR", nap_home);
    // Use gRPC transport for tests — QUIC (lore://) may not work in all
    // environments (e.g. macOS sandbox).  gRPC on TCP 41337 is reliable.
    cmd.env("NAP_LORE_URL_BASE", "grpc://localhost:41337");
    cmd.env("NAP_WORKSPACE_ID", "default");

    cmd
}

#[cfg(feature = "workflow-e2e")]
/// Initialize a provider and repository using `--provider remote`.
///
/// This avoids the port-conflict problem where multiple tests each try to
/// spawn a lore daemon on the same port.
fn init_provider_and_universe(nap_home: &Path, repository: &str) {
    // Configure provider (no daemon startup)
    nap_cmd(nap_home)
        .arg("init")
        .arg("--provider")
        .arg("remote")
        .arg("--remote-url")
        .arg("lore://localhost:41337")
        .arg("--workspace-id")
        .arg("default")
        .assert()
        .success();

    // Create the repository repository
    nap_cmd(nap_home)
        .arg("init")
        .arg(repository)
        .assert()
        .success();
}

#[cfg(feature = "workflow-e2e")]
#[test]
fn test_readme_quick_start_workflow() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let nap_home = tmp.path().join(".nap");
    std::fs::create_dir_all(&nap_home).unwrap();

    // 1. Initialize a repository
    init_provider_and_universe(&nap_home, "starwars");

    // 2. Create entities
    nap_cmd(&nap_home)
        .arg("create")
        .arg("character")
        .arg("lukeskywalker")
        .arg("-u")
        .arg("starwars")
        .arg("-n")
        .arg("Luke Skywalker")
        .assert()
        .success();

    // 3. Set properties
    nap_cmd(&nap_home)
        .arg("set")
        .arg("nap://starwars/character/lukeskywalker")
        .arg("species")
        .arg("human")
        .assert()
        .success();

    // 4. Resolve a manifest
    nap_cmd(&nap_home)
        .arg("resolve")
        .arg("nap://starwars/character/lukeskywalker")
        .assert()
        .success()
        .stdout(predicate::str::contains("Luke Skywalker"));

    // 5. Query a subtree
    nap_cmd(&nap_home)
        .arg("query")
        .arg("nap://starwars/character/lukeskywalker")
        .arg("properties")
        .assert()
        .success()
        .stdout(predicate::str::contains("species"));
}

#[cfg(feature = "workflow-e2e")]
#[test]
fn test_usage_guide_world_building_workflow() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let nap_home = tmp.path().join(".nap");
    std::fs::create_dir_all(&nap_home).unwrap();

    // Initialize repository
    init_provider_and_universe(&nap_home, "myworld");

    // Define world metadata
    nap_cmd(&nap_home)
        .arg("set")
        .arg("nap://myworld/world/myworld")
        .arg("canon_level")
        .arg("canon")
        .assert()
        .success();

    // Create character
    nap_cmd(&nap_home)
        .arg("create")
        .arg("character")
        .arg("captain-rex")
        .arg("-u")
        .arg("myworld")
        .arg("-n")
        .arg("Captain Rex")
        .assert()
        .success();

    // Set character properties
    nap_cmd(&nap_home)
        .arg("set")
        .arg("nap://myworld/character/captain-rex")
        .arg("rank")
        .arg("Captain")
        .assert()
        .success();

    // Commit changes
    nap_cmd(&nap_home)
        .arg("commit")
        .arg("myworld")
        .arg("-m")
        .arg("Complete world-building")
        .arg("-a")
        .arg("writer@studio.com")
        .assert()
        .success();
}
