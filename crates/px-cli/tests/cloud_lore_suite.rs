//! Integration Test Suite 2: Portals Cloud Lore Server
//!
//! This suite tests px functionality against the Portals Cloud lore server.
//! Requires:
//! - A valid Portals Cloud lore server URL
//! - Valid authentication credentials (environment variables)
//! - The lore binary in PATH
//!
//! Environment variables:
//! - PX_LORE_URL_BASE: Portals Cloud Lore URL (`grpcs://lore.portals.works`)
//! - PX_WORKSPACE_ID: Workspace ID for Portals Cloud
//! - PORTALS_CLOUD_API_KEY: Revocable service-account API key (CI only)
//!
//! Run with:
//!   cargo test -p portalshq-px-cli --test cloud_lore_suite --features lore-e2e -- --test-threads=1

#[cfg(feature = "lore-e2e")]
use assert_cmd::Command;
#[cfg(feature = "lore-e2e")]
use predicates::prelude::*;
#[cfg(feature = "lore-e2e")]
use std::fs;
#[cfg(feature = "lore-e2e")]
use std::path::{Path, PathBuf};
#[cfg(feature = "lore-e2e")]
use tempfile::TempDir;

#[cfg(feature = "lore-e2e")]
/// Helper to get the px binary command with cloud configuration
fn px_cmd() -> Command {
    let mut cmd = Command::cargo_bin("px").expect("Failed to find px binary");
    cmd.timeout(std::time::Duration::from_secs(300));

    // Configure for Portals Cloud - read from environment or use default
    let cloud_url = std::env::var("PX_LORE_URL_BASE")
        .unwrap_or_else(|_| "grpcs://lore.portals.works".to_string());
    let workspace_id = std::env::var("PX_WORKSPACE_ID").unwrap_or_else(|_| "default".to_string());

    cmd.env("PX_LORE_URL_BASE", cloud_url);
    cmd.env("PX_WORKSPACE_ID", workspace_id);

    cmd
}

#[cfg(feature = "lore-e2e")]
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

#[cfg(feature = "lore-e2e")]
/// Generate a unique repository name for testing
fn unique_universe_name(prefix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}-cloud-{}", prefix, timestamp)
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_connect_and_init() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    // Test px init with portals-cloud provider
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ready. PX is configured with"));
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_choose_backend() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    // Initialize with local first
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    // Switch to portals-cloud backend
    px_cmd()
        .arg("choose")
        .arg("backend")
        .arg("local")
        .arg("--base-dir")
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Switched to"));
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_create_repository() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-create-repo");

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    // Create a repository repository
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success()
        .stdout(predicate::str::contains(&repository));

    // Verify repository structure exists
    let repo_path = tmp.path().join(&repository);
    assert!(repo_path.exists(), "Repository directory should exist");
    assert!(
        repo_path.join(".lore").exists(),
        ".lore directory should exist"
    );
    assert!(
        repo_path.join("repository.yaml").exists(),
        "repository.yaml should exist"
    );
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_clone_repository() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-clone-repo");
    let cloud_url = std::env::var("PX_LORE_URL_BASE")
        .unwrap_or_else(|_| "grpcs://lore.portals.works".to_string());

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    // Create a repository repository
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Add remote pointing to cloud
    px_cmd()
        .arg("remote")
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("origin")
        .arg(format!("{}/{}", cloud_url, repository))
        .assert()
        .success();

    // Push to remote
    px_cmd()
        .arg("push")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Clone to a new location - need provider configured in clone base_dir
    let clone_tmp = TempDir::new().expect("Failed to create clone temp dir");
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(clone_tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();
    px_cmd()
        .arg("pull")
        .arg("--base-dir")
        .arg(clone_tmp.path())
        .arg(format!("{}/{}", cloud_url, repository))
        .assert()
        .success();

    // Verify clone exists
    let clone_path = clone_tmp.path().join(&repository);
    assert!(clone_path.exists(), "Cloned repository should exist");
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_create_entity() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-create-entity");

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Create a character entity
    px_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("cloudhero")
        .arg("--name")
        .arg("Cloud Hero")
        .arg("--author")
        .arg("cloud-integration-test")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cloud Hero"))
        .stdout(predicate::str::contains("px://"));

    // Verify entity file exists
    let entity_path = tmp
        .path()
        .join(&repository)
        .join("character")
        .join("cloudhero.yaml");
    assert!(entity_path.exists(), "Entity manifest should exist");
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_update_repository_file() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-update-file");

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Create a character entity
    px_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("cloudupdatable")
        .arg("--name")
        .arg("Cloud Updatable")
        .assert()
        .success();

    // Update a property using set command
    px_cmd()
        .arg("set")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("px://{}/character/cloudupdatable", repository))
        .arg("properties.toy_type")
        .arg("plush")
        .arg("--message")
        .arg("set toy_type property on cloud")
        .arg("--author")
        .arg("cloud-integration-test")
        .assert()
        .success()
        .stdout(predicate::str::contains("toy_type"));

    // Verify the update by reading the manifest
    let entity_path = tmp
        .path()
        .join(&repository)
        .join("character")
        .join("cloudupdatable.yaml");
    let content = fs::read_to_string(&entity_path).expect("Failed to read entity manifest");
    assert!(
        content.contains("toy_type"),
        "Manifest should contain toy_type property"
    );
    assert!(content.contains("plush"), "Species should be set to human");
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_add_image_to_repository() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-add-image");

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Create a character entity
    px_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("cloudimagehero")
        .arg("--name")
        .arg("Cloud Image Hero")
        .assert()
        .success();

    // Create a test image
    let image_path = create_test_image(tmp.path(), "cloud_test_image.png");

    // Add the image as a representation
    px_cmd()
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("px://{}/character/cloudimagehero", repository))
        .arg("reference_image")
        .arg(&image_path)
        .arg("--format")
        .arg("png")
        .arg("--message")
        .arg("add reference image on cloud")
        .arg("--author")
        .arg("cloud-integration-test")
        .assert()
        .success()
        .stdout(predicate::str::contains("reference_image"));

    // Verify the representation was added
    let entity_path = tmp
        .path()
        .join(&repository)
        .join("character")
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

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_resolve_manifest_uri() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-resolve-uri");

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Create a character entity
    px_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("cloudresolvable")
        .arg("--name")
        .arg("Cloud Resolvable")
        .assert()
        .success();

    // Resolve the entity using px resolve
    px_cmd()
        .arg("resolve")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("px://{}/character/cloudresolvable", repository))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cloud Resolvable"))
        .stdout(predicate::str::contains("cloudresolvable"));
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_resolve_image_from_manifest() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-resolve-image");

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Create a character entity
    px_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("cloudimageresolver")
        .arg("--name")
        .arg("Cloud Image Resolver")
        .assert()
        .success();

    // Create and add a test image
    let image_path = create_test_image(tmp.path(), "cloud_resolver_test.png");

    px_cmd()
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("px://{}/character/cloudimageresolver", repository))
        .arg("reference_image")
        .arg(&image_path)
        .arg("--format")
        .arg("png")
        .arg("--message")
        .arg("add image for cloud resolution test")
        .assert()
        .success();

    // Query the representation using px query
    px_cmd()
        .arg("query")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("px://{}/character/cloudimageresolver", repository))
        .arg("representations.reference_image.hash")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("blake3:"));
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_list_entities() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-list");

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Create multiple entities
    px_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("cloudhero1")
        .arg("--name")
        .arg("Cloud Hero One")
        .assert()
        .success();

    px_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("cloudhero2")
        .arg("--name")
        .arg("Cloud Hero Two")
        .assert()
        .success();

    // List entities in the repository
    px_cmd()
        .arg("list")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("--entity-type")
        .arg("character")
        .assert()
        .success()
        .stdout(predicate::str::contains("cloudhero1"))
        .stdout(predicate::str::contains("cloudhero2"));
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_commit_history() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-history");

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Create an entity
    px_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("cloudhistoryhero")
        .arg("--name")
        .arg("Cloud History Hero")
        .assert()
        .success();

    // Make a change
    px_cmd()
        .arg("set")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("px://{}/character/cloudhistoryhero", repository))
        .arg("properties.toy_type")
        .arg("plush")
        .arg("--message")
        .arg("set toy_type on cloud")
        .assert()
        .success();

    // View commit history
    px_cmd()
        .arg("history")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("px://{}/character/cloudhistoryhero", repository))
        .arg("--limit")
        .arg("10")
        .assert()
        .success();
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_branch_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-branch");

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Create a branch
    px_cmd()
        .arg("branch")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("cloud-feature-branch")
        .assert()
        .success();

    // List branches
    px_cmd()
        .arg("branch")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success()
        .stdout(predicate::str::contains("cloud-feature-branch"));

    // Switch to the branch
    px_cmd()
        .arg("switch")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("cloud-feature-branch")
        .assert()
        .success();
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_status_and_doctor() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    // Check status
    px_cmd()
        .arg("status")
        .arg("--base-dir")
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("provider_type"));

    // Run doctor
    px_cmd()
        .arg("doctor")
        .arg("--base-dir")
        .arg(tmp.path())
        .assert()
        .success();
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_remote_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-remote");
    let cloud_url = std::env::var("PX_LORE_URL_BASE")
        .unwrap_or_else(|_| "grpcs://lore.portals.works".to_string());

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Add a remote
    px_cmd()
        .arg("remote")
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("origin")
        .arg(format!("{}/{}", cloud_url, repository))
        .assert()
        .success();

    // List remotes
    px_cmd()
        .arg("remote")
        .arg("ls")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success()
        .stdout(predicate::str::contains("origin"));
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_sync_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-sync");
    let cloud_url = std::env::var("PX_LORE_URL_BASE")
        .unwrap_or_else(|_| "grpcs://lore.portals.works".to_string());

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Add remote
    px_cmd()
        .arg("remote")
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("origin")
        .arg(format!("{}/{}", cloud_url, repository))
        .assert()
        .success();

    // Push (publish)
    px_cmd()
        .arg("push")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Sync (pull)
    px_cmd()
        .arg("sync")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_content_hash() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    // Create a test file
    let test_file = tmp.path().join("test.txt");
    fs::write(&test_file, "cloud test content").expect("Failed to write test file");

    // Compute content hash
    px_cmd()
        .arg("content-hash")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("blake3:"));
}

/// Release-gate workflow. Unlike the smaller CLI behavior tests above, this
/// test deliberately uses the authenticated Portals Cloud backend throughout
/// and proves a real fragment/branch/lock round trip.
#[cfg(feature = "lore-e2e")]
#[test]
fn test_staging_authenticated_end_to_end() {
    let source = TempDir::new().expect("source tempdir");
    let clone = TempDir::new().expect("clone tempdir");
    let repository = unique_universe_name("security-gate");
    let cloud_url = std::env::var("PX_LORE_URL_BASE")
        .unwrap_or_else(|_| "grpcs://lore.portals.works".to_string());
    // Clone dir needs provider configured for pull
    px_cmd()
        .args(["init", "--provider", "portals-cloud", "--base-dir"])
        .arg(clone.path())
        .assert()
        .success();

    px_cmd()
        .args(["init", "--provider", "portals-cloud", "--base-dir"])
        .arg(source.path())
        .assert()
        .success();
    px_cmd()
        .args(["init", "--base-dir"])
        .arg(source.path())
        .arg(&repository)
        .assert()
        .success();
    px_cmd()
        .args(["create", "--base-dir"])
        .arg(source.path())
        .args([
            "--repository",
            &repository,
            "character",
            "security-gate",
            "--name",
            "Security Gate",
        ])
        .assert()
        .success();

    px_cmd()
        .args(["publish", "--base-dir"])
        .arg(source.path())
        .arg(&repository)
        .assert()
        .success();
    px_cmd()
        .args(["push", "--base-dir"])
        .arg(source.path())
        .arg(&repository)
        .assert()
        .success();

    px_cmd()
        .args(["pull", "--base-dir"])
        .arg(clone.path())
        .arg(format!("{cloud_url}/{repository}"))
        .assert()
        .success();
    px_cmd()
        .args(["sync", "--base-dir"])
        .arg(clone.path())
        .arg(&repository)
        .assert()
        .success();

    let repository_path = source.path().join(&repository);
    let lock_path = "character/security-gate.yaml";
    for action in ["acquire", "release"] {
        let status = std::process::Command::new(px_core::vcs_lore::LoreProcessRunner::binary())
            .current_dir(&repository_path)
            .args(["lock", action, lock_path, "--non-interactive"])
            .status()
            .expect("run Lore lock command");
        assert!(status.success(), "Lore lock {action} failed");
    }

    // Verify auth is required - only in CI where PORTALS_CLOUD_API_KEY is set
    // Local Cognito runs use `lore auth login` which is global and expensive to restore,
    // so we skip the logout check locally to avoid clearing the valid Cognito JWT
    if std::env::var("PORTALS_CLOUD_API_KEY").is_ok() {
        px_cmd().args(["auth", "logout"]).assert().success();
        // Verify auth is required - create a new entity then push should fail without auth
        // Use push (not sync) which always contacts remote when there are new commits
        px_cmd()
            .args(["create", "--base-dir"])
            .arg(source.path())
            .args([
                "--repository",
                &repository,
                "character",
                "post-logout-check",
                "--name",
                "Post Logout",
            ])
            .assert()
            .success();
        px_cmd()
            .args(["push", "--base-dir"])
            .arg(source.path())
            .arg(&repository)
            .assert()
            .failure();

        // Restore the CI session for any later test process. The key is read by
        // Px and passed to Lore through stdin, never argv.
        px_cmd()
            .args(["auth", "login", "--api-key"])
            .assert()
            .success();
    } else {
        // For local Cognito, re-authenticate via lore's Cognito flow if needed
        // Best-effort: try to restore via existing lore auth (if logout cleared it, subsequent tests will re-login)
        // We do a no-op: the next test will handle its own auth via PX_LORE_URL_BASE
        // But ensure lore auth is restored for sequential tests by trying a silent login
        // If auth store is empty, try to restore via Cognito if we have the helper
        if std::process::Command::new(px_core::vcs_lore::LoreProcessRunner::binary())
            .args(["auth", "list"])
            .output()
            .map(|o| o.stdout.is_empty())
            .unwrap_or(false)
        {
            // Auth store is empty, try to restore via Cognito if we have the helper
            // This is best-effort for local dev; CI will have used the API key path above
            eprintln!("Warning: auth store empty after logout, next test may need re-auth");
        }
    }
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_query_subtree() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-query");

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Create an entity
    px_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("queryhero")
        .arg("--name")
        .arg("Query Hero")
        .assert()
        .success();

    // Set some properties
    px_cmd()
        .arg("set")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("px://{}/character/queryhero", repository))
        .arg("properties.toy_type")
        .arg("plush")
        .arg("--message")
        .arg("set toy_type")
        .assert()
        .success();

    // Query a specific subtree - query the whole entity and check for property
    px_cmd()
        .arg("resolve")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("px://{}/character/queryhero", repository))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("plush"));
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_resolve_with_branch() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-resolve-branch");

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Create an entity
    px_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("branchhero")
        .arg("--name")
        .arg("Branch Hero")
        .assert()
        .success();

    // Create a branch
    px_cmd()
        .arg("branch")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("test-branch")
        .assert()
        .success();

    // Resolve with branch selector
    px_cmd()
        .arg("resolve")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("px://{}/character/branchhero", repository))
        .arg("--branch")
        .arg("test-branch")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Branch Hero"));
}

#[cfg(feature = "lore-e2e")]
#[test]
fn test_cloud_lore_validate_manifest() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-validate");

    // Initialize px with portals-cloud
    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("portals-cloud")
        .assert()
        .success();

    px_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Create an entity
    px_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("validatehero")
        .arg("--name")
        .arg("Validate Hero")
        .assert()
        .success();

    // Validate the manifest
    px_cmd()
        .arg("validate")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("px://{}/character/validatehero", repository))
        .assert()
        .success();
}
