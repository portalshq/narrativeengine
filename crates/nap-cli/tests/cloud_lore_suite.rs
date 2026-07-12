//! Integration Test Suite 2: Portals Cloud Lore Server
//!
//! This suite tests nap functionality against the Portals Cloud lore server.
//! Requires:
//! - A valid Portals Cloud lore server URL
//! - Valid authentication credentials (environment variables)
//! - The lore binary in PATH
//!
//! Environment variables:
//! - NAP_LORE_URL_BASE: Portals Cloud lore server URL (e.g., lore://cloud.portals.ai)
//! - NAP_WORKSPACE_ID: Workspace ID for Portals Cloud
//! - PORTALS_CLOUD_AUTH_TOKEN: Authentication token (if required)
//!
//! Run with:
//!   cargo test -p nap-cli --test cloud_lore_suite --features lore-e2e -- --test-threads=1

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Helper to get the nap binary command with cloud configuration
fn nap_cmd() -> Command {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.timeout(std::time::Duration::from_secs(180));

    // Configure for Portals Cloud - read from environment or use default
    let cloud_url = std::env::var("NAP_LORE_URL_BASE")
        .unwrap_or_else(|_| "lore://cloud.portals.ai".to_string());
    let workspace_id = std::env::var("NAP_WORKSPACE_ID").unwrap_or_else(|_| "default".to_string());

    cmd.env("NAP_LORE_URL_BASE", cloud_url);
    cmd.env("NAP_WORKSPACE_ID", workspace_id);

    // Add auth token if available
    if let Ok(auth_token) = std::env::var("PORTALS_CLOUD_AUTH_TOKEN") {
        cmd.env("PORTALS_CLOUD_AUTH_TOKEN", auth_token);
    }

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
    format!("{}-cloud-{}", prefix, timestamp)
}

#[test]
fn test_cloud_lore_connect_and_init() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    // Test nap init with portals-cloud provider
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized NAP"));
}

#[test]
fn test_cloud_lore_choose_backend() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    // Initialize with local first
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    // Switch to portals-cloud backend
    nap_cmd()
        .arg("choose")
        .arg("backend")
        .arg("--provider")
        .arg("portals-cloud")
        .arg("--base-dir")
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Switched to"));
}

#[test]
fn test_cloud_lore_create_repository() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-create-repo");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
fn test_cloud_lore_clone_repository() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-clone-repo");
    let cloud_url = std::env::var("NAP_LORE_URL_BASE")
        .unwrap_or_else(|_| "lore://cloud.portals.ai".to_string());

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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

    // Add remote pointing to cloud
    nap_cmd()
        .arg("remote")
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .arg("origin")
        .arg(format!("{}/{}", cloud_url, universe))
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
        .arg(format!("{}/{}", cloud_url, universe))
        .assert()
        .success();

    // Verify clone exists
    let clone_path = clone_tmp.path().join(&universe);
    assert!(clone_path.exists(), "Cloned repository should exist");
}

#[test]
fn test_cloud_lore_create_entity() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-create-entity");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg("cloudhero")
        .arg("--name")
        .arg("Cloud Hero")
        .arg("--author")
        .arg("cloud-integration-test")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cloud Hero"))
        .stdout(predicate::str::contains("nap://"));

    // Verify entity file exists
    let entity_path = tmp
        .path()
        .join(&universe)
        .join("characters")
        .join("cloudhero.yaml");
    assert!(entity_path.exists(), "Entity manifest should exist");
}

#[test]
fn test_cloud_lore_update_repository_file() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-update-file");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg("cloudupdatable")
        .arg("--name")
        .arg("Cloud Updatable")
        .assert()
        .success();

    // Update a property using set command
    nap_cmd()
        .arg("set")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/cloudupdatable", universe))
        .arg("properties.species")
        .arg("human")
        .arg("--message")
        .arg("set species property on cloud")
        .arg("--author")
        .arg("cloud-integration-test")
        .assert()
        .success()
        .stdout(predicate::str::contains("species"));

    // Verify the update by reading the manifest
    let entity_path = tmp
        .path()
        .join(&universe)
        .join("characters")
        .join("cloudupdatable.yaml");
    let content = fs::read_to_string(&entity_path).expect("Failed to read entity manifest");
    assert!(
        content.contains("species"),
        "Manifest should contain species property"
    );
    assert!(content.contains("human"), "Species should be set to human");
}

#[test]
fn test_cloud_lore_add_image_to_repository() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-add-image");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg("cloudimagehero")
        .arg("--name")
        .arg("Cloud Image Hero")
        .assert()
        .success();

    // Create a test image
    let image_path = create_test_image(tmp.path(), "cloud_test_image.png");

    // Add the image as a representation
    nap_cmd()
        .arg("add-repr")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/cloudimagehero", universe))
        .arg("reference_image")
        .arg("--file")
        .arg(&image_path)
        .arg("--format")
        .arg("png")
        .arg("--message")
        .arg("add reference image on cloud")
        .arg("--author")
        .arg("cloud-integration-test")
        .assert()
        .success()
        .stdout(predicate::str::contains("reference_image"))
        .stdout(predicate::str::contains("blake3:"));

    // Verify the representation was added
    let entity_path = tmp
        .path()
        .join(&universe)
        .join("characters")
        .join("cloudimagehero.yaml");
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
fn test_cloud_lore_resolve_manifest_uri() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-resolve-uri");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg("cloudresolvable")
        .arg("--name")
        .arg("Cloud Resolvable")
        .assert()
        .success();

    // Resolve the entity using nap resolve
    nap_cmd()
        .arg("resolve")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/cloudresolvable", universe))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cloud Resolvable"))
        .stdout(predicate::str::contains("cloudresolvable"));
}

#[test]
fn test_cloud_lore_resolve_image_from_manifest() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-resolve-image");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg("cloudimageresolver")
        .arg("--name")
        .arg("Cloud Image Resolver")
        .assert()
        .success();

    // Create and add a test image
    let image_path = create_test_image(tmp.path(), "cloud_resolver_test.png");

    nap_cmd()
        .arg("add-repr")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/cloudimageresolver", universe))
        .arg("reference_image")
        .arg("--file")
        .arg(&image_path)
        .arg("--format")
        .arg("png")
        .arg("--message")
        .arg("add image for cloud resolution test")
        .assert()
        .success();

    // Query the representation using nap query
    nap_cmd()
        .arg("query")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/cloudimageresolver", universe))
        .arg("representations.reference_image.hash")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("blake3:"));
}

#[test]
fn test_cloud_lore_list_entities() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-list");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg("cloudhero1")
        .arg("--name")
        .arg("Cloud Hero One")
        .assert()
        .success();

    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--universe")
        .arg(&universe)
        .arg("character")
        .arg("cloudhero2")
        .arg("--name")
        .arg("Cloud Hero Two")
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
        .stdout(predicate::str::contains("cloudhero1"))
        .stdout(predicate::str::contains("cloudhero2"));
}

#[test]
fn test_cloud_lore_commit_history() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-history");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg("cloudhistoryhero")
        .arg("--name")
        .arg("Cloud History Hero")
        .assert()
        .success();

    // Make a change
    nap_cmd()
        .arg("set")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/cloudhistoryhero", universe))
        .arg("properties.species")
        .arg("human")
        .arg("--message")
        .arg("set species on cloud")
        .assert()
        .success();

    // View commit history
    nap_cmd()
        .arg("history")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/cloudhistoryhero", universe))
        .arg("--limit")
        .arg("10")
        .assert()
        .success();
}

#[test]
fn test_cloud_lore_branch_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-branch");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg("cloud-feature-branch")
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
        .stdout(predicate::str::contains("cloud-feature-branch"));

    // Switch to the branch
    nap_cmd()
        .arg("switch")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .arg("cloud-feature-branch")
        .assert()
        .success();
}

#[test]
fn test_cloud_lore_tag_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-tag");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg("cloud-v1.0.0")
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
        .stdout(predicate::str::contains("cloud-v1.0.0"));
}

#[test]
fn test_cloud_lore_status_and_doctor() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
fn test_cloud_lore_remote_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-remote");
    let cloud_url = std::env::var("NAP_LORE_URL_BASE")
        .unwrap_or_else(|_| "lore://cloud.portals.ai".to_string());

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg(format!("{}/{}", cloud_url, universe))
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
fn test_cloud_lore_sync_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-sync");
    let cloud_url = std::env::var("NAP_LORE_URL_BASE")
        .unwrap_or_else(|_| "lore://cloud.portals.ai".to_string());

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg(format!("{}/{}", cloud_url, universe))
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
fn test_cloud_lore_content_hash() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    // Create a test file
    let test_file = tmp.path().join("test.txt");
    fs::write(&test_file, "cloud test content").expect("Failed to write test file");

    // Compute content hash
    nap_cmd()
        .arg("content-hash")
        .arg("--file")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("blake3:"));
}

#[test]
fn test_cloud_lore_query_subtree() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-query");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg("queryhero")
        .arg("--name")
        .arg("Query Hero")
        .assert()
        .success();

    // Set some properties
    nap_cmd()
        .arg("set")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/queryhero", universe))
        .arg("properties.species")
        .arg("human")
        .arg("--message")
        .arg("set species")
        .assert()
        .success();

    // Query a specific subtree
    nap_cmd()
        .arg("query")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/queryhero", universe))
        .arg("properties.species")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("human"));
}

#[test]
fn test_cloud_lore_resolve_with_branch() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-resolve-branch");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg("branchhero")
        .arg("--name")
        .arg("Branch Hero")
        .assert()
        .success();

    // Create a branch
    nap_cmd()
        .arg("branch")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&universe)
        .arg("test-branch")
        .assert()
        .success();

    // Resolve with branch selector
    nap_cmd()
        .arg("resolve")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/branchhero", universe))
        .arg("--branch")
        .arg("test-branch")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Branch Hero"));
}

#[test]
fn test_cloud_lore_validate_manifest() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let universe = unique_universe_name("test-validate");

    // Initialize nap with portals-cloud
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
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
        .arg("validatehero")
        .arg("--name")
        .arg("Validate Hero")
        .assert()
        .success();

    // Validate the manifest
    nap_cmd()
        .arg("validate")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--uri")
        .arg(format!("nap://{}/character/validatehero", universe))
        .assert()
        .success();
}
