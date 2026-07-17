//! Integration Test Suite: Documentation Workflows
//!
//! This suite tests nap functionality against a local lore server,
//! following the "Quick Start" and "Usage Guide" workflows from documentation.
//!
//! Run with:
//!   cargo test -p nap-cli --test workflow_integration_suite --features lore-e2e -- --test-threads=1

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Helper to get the nap binary command, correctly configured for testing
fn nap_cmd(nap_home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.timeout(std::time::Duration::from_secs(120));
    
    // Set NAP_DIR to the temporary home directory to isolate the config
    cmd.env("NAP_DIR", nap_home);
    cmd.env("NAP_LORE_URL_BASE", "lore://localhost:41337");
    cmd.env("NAP_WORKSPACE_ID", "default");
    
    cmd
}

#[test]
fn test_readme_quick_start_workflow() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let nap_home = tmp.path().join(".nap");
    std::fs::create_dir_all(&nap_home).unwrap();

    // 1. Initialize a universe
    nap_cmd(&nap_home)
        .arg("init")
        .arg("starwars")
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

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

#[test]
fn test_usage_guide_world_building_workflow() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let nap_home = tmp.path().join(".nap");
    std::fs::create_dir_all(&nap_home).unwrap();

    // Initialize universe
    nap_cmd(&nap_home)
        .arg("init")
        .arg("myworld")
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

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
