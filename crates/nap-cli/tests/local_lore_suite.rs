//! Integration Test Suite 1: Local Lore Server
//!
//! This suite tests nap functionality against a local lore server.
//! Requires:
//! - A running local lore server at lore://localhost:41337
//! - The lore binary in PATH
//! - Environment: NAP_LORE_URL_BASE=lore://localhost:41337
//!
//! Run with:
//!   cargo test -p nap-cli --test local_lore_suite --features lore-e2e -- --test-threads=1

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Helper to get the nap binary command
fn nap_cmd() -> Command {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.timeout(std::time::Duration::from_secs(120));
    cmd.env("NAP_LORE_URL_BASE", "lore://localhost:41337");
    cmd.env("NAP_WORKSPACE_ID", "default");
    cmd
}

/// Helper to create a test image file
fn create_test_image(dir: &Path, name: &str) -> PathBuf {
    let image_path = dir.join(name);
    // Create a minimal PNG file (1x1 transparent pixel)
    let png_data: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
        0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, // width: 1
        0x00, 0x00, 0x00, 0x01, // height: 1
        0x08, 0x06, 0x00, 0x00, 0x00, // bit depth, color type, compression, filter, interlace
        0x1F, 0x15, 0xC4, 0x89, // CRC
        0x00, 0x00, 0x00, 0x0A, // IDAT chunk length
        0x49, 0x44, 0x41, 0x54, // IDAT
        0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, // compressed data
        0x0D, 0x0A, 0x2D, 0xB4, // CRC
        0x00, 0x00, 0x00, 0x00, // IEND chunk length
        0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82, // CRC
    ];
    fs::write(&image_path, png_data).expect("Failed to write test image");
    image_path
}

/// Generate a unique universe name for testing
fn unique_universe_name(prefix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}-{}", prefix, timestamp)
}

#[test]
fn test_local_lore_connect_and_init() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    // Test nap init with local provider
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized NAP"));
}

#[test]
fn test_local_lore_create_repository() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-create-repo");

    // Initialize nap
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    // Create a universe repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success()
        .stdout(predicate::str::contains(&universe));

    // Verify repository structure exists
    let repo_path = tmp.path().join(&universe);
    assert!(repo_path.exists(), "Repository directory should exist");
    assert!(
        repo_path.join(".nap").exists(),
        ".nap directory should exist"
    );
    assert!(
        repo_path.join("universe.yaml").exists(),
        "universe.yaml should exist"
    );
}

#[test]
fn test_local_lore_clone_repository() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-clone-repo");

    // Initialize nap
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    // Create a universe repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Add remote
    nap_cmd()
        .arg("remote")
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .arg("origin")
        .arg(format!("lore://localhost:41337/{}", universe))
        .assert()
        .success();

    // Push to remote
    nap_cmd()
        .arg("push")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Clone to a new location
    let clone_tmp = TempDir::new().expect("Failed to create clone temp dir");
    nap_cmd()
        .arg("pull")
        .arg("--base-dir")
        .arg(clone_tmp.path())
        .arg(format!("lore://localhost:41337/{}", universe))
        .assert()
        .success();

    // Verify clone exists
    let clone_path = clone_tmp.path().join(&universe);
    assert!(clone_path.exists(), "Cloned repository should exist");
}

#[test]
fn test_local_lore_create_entity() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-create-entity");

    // Initialize nap and create repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Create a character entity
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--universe")
        .arg(&universe)
        .arg("character")
        .arg("testhero")
        .arg("--name")
        .arg("Test Hero")
        .arg("--author")
        .arg("integration-test")
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Hero"))
        .stdout(predicate::str::contains("nap://"));

    // Verify entity file exists
    let entity_path = tmp
        .path()
        .join(&universe)
        .join("characters")
        .join("testhero.yaml");
    assert!(entity_path.exists(), "Entity manifest should exist");
}

#[test]
fn test_local_lore_update_repository_file() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-update-file");

    // Initialize nap and create repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Create a character entity
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--universe")
        .arg(&universe)
        .arg("character")
        .arg("updatablehero")
        .arg("--name")
        .arg("Updatable Hero")
        .assert()
        .success();

    // Update a property using set command
    nap_cmd()
        .arg("set")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/updatablehero", universe))
        .arg("properties.species")
        .arg("human")
        .arg("--message")
        .arg("set species property")
        .arg("--author")
        .arg("integration-test")
        .assert()
        .success()
        .stdout(predicate::str::contains("species"));

    // Verify the update by reading the manifest
    let entity_path = tmp
        .path()
        .join(&universe)
        .join("characters")
        .join("updatablehero.yaml");
    let content = fs::read_to_string(&entity_path).expect("Failed to read entity manifest");
    assert!(
        content.contains("species"),
        "Manifest should contain species property"
    );
    assert!(content.contains("human"), "Species should be set to human");
}

#[test]
fn test_local_lore_add_image_to_repository() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-add-image");

    // Initialize nap and create repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Create a character entity
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--universe")
        .arg(&universe)
        .arg("character")
        .arg("imagehero")
        .arg("--name")
        .arg("Image Hero")
        .assert()
        .success();

    // Create a test image
    let image_path = create_test_image(tmp.path(), "test_image.png");

    // Add the image as a representation
    nap_cmd()
        .arg("add-repr")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/imagehero", universe))
        .arg("reference_image")
        .arg("--file")
        .arg(&image_path)
        .arg("--format")
        .arg("png")
        .arg("--message")
        .arg("add reference image")
        .arg("--author")
        .arg("integration-test")
        .assert()
        .success()
        .stdout(predicate::str::contains("reference_image"))
        .stdout(predicate::str::contains("blake3:"));

    // Verify the representation was added
    let entity_path = tmp
        .path()
        .join(&universe)
        .join("characters")
        .join("imagehero.yaml");
    let content = fs::read_to_string(&entity_path).expect("Failed to read entity manifest");
    assert!(
        content.contains("reference_image"),
        "Manifest should contain reference_image"
    );
    assert!(
        content.contains("blake3:"),
        "Manifest should contain content hash"
    );
}

#[test]
fn test_local_lore_resolve_manifest_uri() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-resolve-uri");

    // Initialize nap and create repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Create a character entity
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--universe")
        .arg(&universe)
        .arg("character")
        .arg("resolvablehero")
        .arg("--name")
        .arg("Resolvable Hero")
        .assert()
        .success();

    // Resolve the entity using nap resolve
    nap_cmd()
        .arg("resolve")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/resolvablehero", universe))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolvable Hero"))
        .stdout(predicate::str::contains("resolvablehero"));
}

#[test]
fn test_local_lore_resolve_image_from_manifest() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-resolve-image");

    // Initialize nap and create repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Create a character entity
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--universe")
        .arg(&universe)
        .arg("character")
        .arg("imageresolver")
        .arg("--name")
        .arg("Image Resolver")
        .assert()
        .success();

    // Create and add a test image
    let image_path = create_test_image(tmp.path(), "resolver_test.png");

    nap_cmd()
        .arg("add-repr")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/imageresolver", universe))
        .arg("reference_image")
        .arg("--file")
        .arg(&image_path)
        .arg("--format")
        .arg("png")
        .arg("--message")
        .arg("add image for resolution test")
        .assert()
        .success();

    // Query the representation using nap query
    nap_cmd()
        .arg("query")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/imageresolver", universe))
        .arg("representations.reference_image.hash")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("blake3:"));
}

#[test]
fn test_local_lore_list_entities() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-list");

    // Initialize nap and create repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Create multiple entities
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--universe")
        .arg(&universe)
        .arg("character")
        .arg("hero1")
        .arg("--name")
        .arg("Hero One")
        .assert()
        .success();

    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--universe")
        .arg(&universe)
        .arg("character")
        .arg("hero2")
        .arg("--name")
        .arg("Hero Two")
        .assert()
        .success();

    // List entities in the universe
    nap_cmd()
        .arg("list")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--universe")
        .arg(&universe)
        .arg("--entity-type")
        .arg("character")
        .assert()
        .success()
        .stdout(predicate::str::contains("hero1"))
        .stdout(predicate::str::contains("hero2"));
}

#[test]
fn test_local_lore_commit_history() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-history");

    // Initialize nap and create repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Create an entity
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--universe")
        .arg(&universe)
        .arg("character")
        .arg("historyhero")
        .arg("--name")
        .arg("History Hero")
        .assert()
        .success();

    // Make a change
    nap_cmd()
        .arg("set")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/historyhero", universe))
        .arg("properties.species")
        .arg("human")
        .arg("--message")
        .arg("set species")
        .assert()
        .success();

    // View commit history
    nap_cmd()
        .arg("history")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/historyhero", universe))
        .arg("--limit")
        .arg("10")
        .assert()
        .success();
}

#[test]
fn test_local_lore_branch_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-branch");

    // Initialize nap and create repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Create a branch
    nap_cmd()
        .arg("branch")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .arg("feature-branch")
        .assert()
        .success();

    // List branches
    nap_cmd()
        .arg("branch")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success()
        .stdout(predicate::str::contains("feature-branch"));

    // Switch to the branch
    nap_cmd()
        .arg("switch")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .arg("feature-branch")
        .assert()
        .success();
}

#[test]
fn test_local_lore_tag_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-tag");

    // Initialize nap and create repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Create a tag
    nap_cmd()
        .arg("tag")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .arg("v1.0.0")
        .assert()
        .success();

    // List tags
    nap_cmd()
        .arg("tag")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success()
        .stdout(predicate::str::contains("v1.0.0"));
}

#[test]
fn test_local_lore_status_and_doctor() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    // Initialize nap
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    // Check status
    nap_cmd()
        .arg("status")
        .arg("--base-dir")
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Provider"));

    // Run doctor
    nap_cmd()
        .arg("doctor")
        .arg("--base-dir")
        .arg(tmp.path())
        .assert()
        .success();
}

#[test]
fn test_local_lore_remote_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-remote");

    // Initialize nap and create repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Add a remote
    nap_cmd()
        .arg("remote")
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .arg("origin")
        .arg(format!("lore://localhost:41337/{}", universe))
        .assert()
        .success();

    // List remotes
    nap_cmd()
        .arg("remote")
        .arg("ls")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success()
        .stdout(predicate::str::contains("origin"));
}

#[test]
fn test_local_lore_sync_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-sync");

    // Initialize nap and create repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Add remote
    nap_cmd()
        .arg("remote")
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .arg("origin")
        .arg(format!("lore://localhost:41337/{}", universe))
        .assert()
        .success();

    // Push (publish)
    nap_cmd()
        .arg("push")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();

    // Sync (pull)
    nap_cmd()
        .arg("sync")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .assert()
        .success();
}

#[test]
fn test_local_lore_content_hash() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    // Create a test file
    let test_file = tmp.path().join("test.txt");
    fs::write(&test_file, "test content").expect("Failed to write test file");

    // Compute content hash
    nap_cmd()
        .arg("content-hash")
        .arg("--file")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("blake3:"));
}
