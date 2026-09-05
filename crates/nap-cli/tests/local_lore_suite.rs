//! Integration Test Suite 1: Local Lore Server
//!
//! This suite tests nap functionality against a local lore server.
//! Requires:
//! - A running local lore server at lore://localhost:41337
//! - The lore binary in PATH
//! - Environment: NAP_LORE_URL_BASE=lore://localhost:41337
//!
//! Run with:
//!   cargo test -p nap-cli --test local_lore_suite --features local-e2e -- --test-threads=1

#[cfg(feature = "local-e2e")]
use assert_cmd::Command;
#[cfg(feature = "local-e2e")]
use predicates::prelude::*;
#[cfg(feature = "local-e2e")]
use std::fs;
#[cfg(feature = "local-e2e")]
use std::path::{Path, PathBuf};
#[cfg(feature = "local-e2e")]
use tempfile::TempDir;

#[cfg(feature = "local-e2e")]
/// Helper to get the nap binary command
fn nap_cmd() -> Command {
    let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
    cmd.timeout(std::time::Duration::from_secs(300));
    cmd.env("NAP_LORE_URL_BASE", "lore://localhost:41337");
    cmd.env("NAP_WORKSPACE_ID", "default");
    cmd
}

#[cfg(feature = "local-e2e")]
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

#[cfg(feature = "local-e2e")]
/// Create a representation that Lore stores as a fragment tree rather than a
/// single raw-content fragment.
fn create_fragmented_test_file(dir: &Path, name: &str) -> PathBuf {
    let file_path = dir.join(name);
    fs::write(&file_path, vec![0xA5; 256 * 1024 + 1])
        .expect("Failed to write fragmented test file");
    file_path
}

#[cfg(feature = "local-e2e")]
/// Generate a unique repository name for testing
fn unique_universe_name(prefix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}-{}", prefix, timestamp)
}

#[cfg(feature = "local-e2e")]
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
        .stdout(predicate::str::contains("Ready. NAP is configured with"));
}

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_create_repository() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-create-repo");

    // Initialize nap
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    // Create a repository repository
    nap_cmd()
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

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_clone_repository() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-clone-repo");

    // Initialize nap
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--provider")
        .arg("local")
        .assert()
        .success();

    // Create a repository repository
    nap_cmd()
        .arg("init")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Add remote
    nap_cmd()
        .arg("remote")
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("origin")
        .arg(format!("lore://localhost:41337/{}", repository))
        .assert()
        .success();

    // Push to remote
    nap_cmd()
        .arg("push")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    // Clone to a new location
    let clone_tmp = TempDir::new().expect("Failed to create clone temp dir");
    nap_cmd()
        .arg("pull")
        .arg("--base-dir")
        .arg(clone_tmp.path())
        .arg(format!("lore://localhost:41337/{}", repository))
        .assert()
        .success();

    // Verify clone exists
    let clone_path = clone_tmp.path().join(&repository);
    assert!(clone_path.exists(), "Cloned repository should exist");
}

/// A sparse entity pull must retrieve both NAP markers from one remote
/// revision: the repository manifest and the requested entity manifest.
#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_pull_entity_materializes_manifests() {
    let source = TempDir::new().expect("Failed to create source temp dir");
    let repository = unique_universe_name("test-pull-entity");

    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(source.path())
        .args(["--provider", "local"])
        .assert()
        .success();
    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(source.path())
        .arg(&repository)
        .assert()
        .success();
    nap_cmd()
        .args(["create", "--base-dir"])
        .arg(source.path())
        .args(["--repository", &repository, "character", "claire-cole"])
        .args(["--name", "Claire Cole"])
        .assert()
        .success();
    nap_cmd()
        .args(["remote", "add", "--base-dir"])
        .arg(source.path())
        .args([&repository, "origin"])
        .arg(format!("lore://localhost:41337/{repository}"))
        .assert()
        .success();
    nap_cmd()
        .args(["push", "--base-dir"])
        .arg(source.path())
        .arg(&repository)
        .assert()
        .success();

    let destination = TempDir::new().expect("Failed to create destination temp dir");
    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(destination.path())
        .args(["--provider", "local"])
        .assert()
        .success();
    nap_cmd()
        .args(["pull", "--base-dir"])
        .arg(destination.path())
        .arg(format!("{repository}/character/claire-cole"))
        .assert()
        .success();

    let pulled = destination.path().join(&repository);
    assert!(pulled.join("repository.yaml").is_file());
    assert!(pulled.join("character/claire-cole.yaml").is_file());
}

/// Server-backed list commands must run their gRPC client inside NAP's Tokio
/// bridge. This regresses the historical "there is no reactor running"
/// panic from hyper-util.
#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_list_remote_repository_and_entities() {
    let source = TempDir::new().expect("Failed to create source temp dir");
    let repository = unique_universe_name("test-list-remote");

    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(source.path())
        .args(["--provider", "local"])
        .assert()
        .success();
    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(source.path())
        .arg(&repository)
        .assert()
        .success();
    nap_cmd()
        .args(["create", "--base-dir"])
        .arg(source.path())
        .args(["--repository", &repository, "character", "claire-cole"])
        .args(["--name", "Claire Cole"])
        .assert()
        .success();
    nap_cmd()
        .args(["remote", "add", "--base-dir"])
        .arg(source.path())
        .args([&repository, "origin"])
        .arg(format!("lore://localhost:41337/{repository}"))
        .assert()
        .success();
    nap_cmd()
        .args(["push", "--base-dir"])
        .arg(source.path())
        .arg(&repository)
        .assert()
        .success();

    let reader = TempDir::new().expect("Failed to create reader temp dir");
    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(reader.path())
        .args(["--provider", "local"])
        .assert()
        .success();
    nap_cmd()
        .args(["list", "--base-dir"])
        .arg(reader.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&repository));
    nap_cmd()
        .args(["list", "--base-dir"])
        .arg(reader.path())
        .arg(&repository)
        .assert()
        .success()
        .stdout(predicate::str::contains("claire-cole"));
}

/// Exercises every server-backed read command against one pushed repository.
/// These commands default to the configured Lore server and must never depend
/// on a local checkout in the reader NAP home.
#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_remote_read_command_suite() {
    let source = TempDir::new().expect("Failed to create source temp dir");
    let repository = unique_universe_name("test-remote-reads");
    let uri = format!("{repository}/character/claire-cole");

    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(source.path())
        .args(["--provider", "local"])
        .assert()
        .success();
    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(source.path())
        .arg(&repository)
        .assert()
        .success();
    nap_cmd()
        .args(["create", "--base-dir"])
        .arg(source.path())
        .args(["--repository", &repository, "character", "claire-cole"])
        .args(["--name", "Claire Cole"])
        .assert()
        .success();
    nap_cmd()
        .args(["remote", "add", "--base-dir"])
        .arg(source.path())
        .args([&repository, "origin"])
        .arg(format!("lore://localhost:41337/{repository}"))
        .assert()
        .success();
    nap_cmd()
        .args(["push", "--base-dir"])
        .arg(source.path())
        .arg(&repository)
        .assert()
        .success();

    let reader = TempDir::new().expect("Failed to create reader temp dir");
    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(reader.path())
        .args(["--provider", "local"])
        .assert()
        .success();

    nap_cmd()
        .args(["resolve", "--base-dir"])
        .arg(reader.path())
        .arg(&uri)
        .assert()
        .success()
        .stdout(predicate::str::contains("Claire Cole"));
    nap_cmd()
        .args(["query", "--base-dir"])
        .arg(reader.path())
        .args([&uri, "name"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Claire Cole"));
    nap_cmd()
        .args(["list", "--base-dir"])
        .arg(reader.path())
        .arg(&repository)
        .assert()
        .success()
        .stdout(predicate::str::contains("claire-cole"));
    nap_cmd()
        .args(["history", "--base-dir"])
        .arg(reader.path())
        .arg(&uri)
        .assert()
        .success();
    nap_cmd()
        .args(["branch", "--base-dir"])
        .arg(reader.path())
        .arg(&repository)
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
    nap_cmd()
        .args(["head-hash", "--base-dir"])
        .arg(reader.path())
        .arg(&repository)
        .assert()
        .success()
        .stdout(predicate::str::contains("hash"));
}

/// A manifest is a file, while Lore's RevisionTree RPC accepts directory
/// prefixes only. This exercises NAP's remote resolver against a real Lore
/// service, proving it reads the manifest through its parent directory.
#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_remote_resolve_reads_manifest_from_parent_tree() {
    let source = TempDir::new().expect("Failed to create source temp dir");
    let repository = unique_universe_name("test-resolve-parent-tree");

    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(source.path())
        .args(["--provider", "local"])
        .assert()
        .success();
    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(source.path())
        .arg(&repository)
        .assert()
        .success();
    nap_cmd()
        .args(["create", "--base-dir"])
        .arg(source.path())
        .args(["--repository", &repository, "character", "nathan-gunn"])
        .args(["--name", "Nathan Gunn"])
        .assert()
        .success();
    nap_cmd()
        .args(["remote", "add", "--base-dir"])
        .arg(source.path())
        .args([&repository, "origin"])
        .arg(format!("lore://localhost:41337/{repository}"))
        .assert()
        .success();
    nap_cmd()
        .args(["push", "--base-dir"])
        .arg(source.path())
        .arg(&repository)
        .assert()
        .success();

    let reader = TempDir::new().expect("Failed to create reader temp dir");
    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(reader.path())
        .args(["--provider", "local"])
        .assert()
        .success();

    nap_cmd()
        .args(["--remote", "resolve", "--base-dir"])
        .arg(reader.path())
        .arg(format!("nap://{repository}/character/nathan-gunn"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Nathan Gunn"));
}

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_create_entity() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-create-entity");

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
        .arg(&repository)
        .assert()
        .success();

    // Create a character entity
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
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
        .join(&repository)
        .join("character")
        .join("testhero.yaml");
    assert!(entity_path.exists(), "Entity manifest should exist");
}

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_update_repository_file() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-update-file");

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
        .arg(&repository)
        .assert()
        .success();

    // Create a character entity
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
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
        .arg(format!("nap://{}/character/updatablehero", repository))
        .arg("properties.toy_type")
        .arg("plush")
        .arg("--message")
        .arg("set toy_type property")
        .arg("--author")
        .arg("integration-test")
        .assert()
        .success()
        .stdout(predicate::str::contains("toy_type"));

    // Verify the update by reading the manifest
    let entity_path = tmp
        .path()
        .join(&repository)
        .join("character")
        .join("updatablehero.yaml");
    let content = fs::read_to_string(&entity_path).expect("Failed to read entity manifest");
    assert!(
        content.contains("toy_type"),
        "Manifest should contain toy_type property"
    );
    assert!(content.contains("plush"), "Species should be set to human");
}

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_presign_redeems_binary_representation() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-presign");

    nap_cmd()
        .args(["init", "--provider", "local", "--base-dir"])
        .arg(tmp.path())
        .assert()
        .success();
    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();
    nap_cmd()
        .args(["create", "--base-dir"])
        .arg(tmp.path())
        .args([
            "--repository",
            &repository,
            "character",
            "testhero",
            "--name",
            "Test Hero",
        ])
        .assert()
        .success();

    // This exceeds Lore's 256 KiB single-fragment threshold. Its Lore address
    // hashes the fragment tree, while NAP's representation hash covers the
    // full file bytes.
    let asset = create_fragmented_test_file(tmp.path(), "presigned.bin");
    let uri = format!("nap://{repository}/character/testhero");
    nap_cmd()
        .args(["add", "--base-dir"])
        .arg(tmp.path())
        .arg(&uri)
        .arg("reference_image")
        .arg(&asset)
        .args(["--format", "bin"])
        .assert()
        .success();
    nap_cmd()
        .args(["push", "--base-dir"])
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    let output = nap_cmd()
        .args(["presign", "--base-dir"])
        .arg(tmp.path())
        .arg(&uri)
        .arg("reference_image")
        .args(["--branch", "main", "--ttl-seconds", "60"])
        .output()
        .expect("Failed to run nap presign");
    assert!(
        output.status.success(),
        "nap presign failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("presign output should be JSON when captured");
    let url = response["url"].as_str().expect("presign output URL");
    let redeemed = std::process::Command::new("curl")
        .args(["--fail", "--silent", "--show-error", url])
        .output()
        .expect("curl is required for the local presign integration test");
    assert!(
        redeemed.status.success(),
        "presigned URL redemption failed: {}",
        String::from_utf8_lossy(&redeemed.stderr)
    );
    assert_eq!(redeemed.stdout, fs::read(asset).unwrap());
}

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_add_image_to_repository() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-add-image");

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
        .arg(&repository)
        .assert()
        .success();

    // Create a character entity
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
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
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/imagehero", repository))
        .arg("reference_image")
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
        .join(&repository)
        .join("character")
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

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_resolve_manifest_uri() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-resolve-uri");

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
        .arg(&repository)
        .assert()
        .success();

    // Create a character entity
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
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
        .arg(format!("nap://{}/character/resolvablehero", repository))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolvable Hero"))
        .stdout(predicate::str::contains("resolvablehero"));
}

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_resolve_image_from_manifest() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-resolve-image");

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
        .arg(&repository)
        .assert()
        .success();

    // Create a character entity
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("imageresolver")
        .arg("--name")
        .arg("Image Resolver")
        .assert()
        .success();

    // Create and add a test image
    let image_path = create_test_image(tmp.path(), "resolver_test.png");

    nap_cmd()
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/imageresolver", repository))
        .arg("reference_image")
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
        .arg(format!("nap://{}/character/imageresolver", repository))
        .arg("representations.reference_image.hash")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("blake3:"));
}

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_list_entities() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-list");

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
        .arg(&repository)
        .assert()
        .success();

    // Create multiple entities
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
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
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("hero2")
        .arg("--name")
        .arg("Hero Two")
        .assert()
        .success();

    // List entities in the repository
    nap_cmd()
        .arg("list")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("--entity-type")
        .arg("character")
        .assert()
        .success()
        .stdout(predicate::str::contains("hero1"))
        .stdout(predicate::str::contains("hero2"));
}

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_commit_history() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-history");

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
        .arg(&repository)
        .assert()
        .success();

    // Create an entity
    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
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
        .arg(format!("nap://{}/character/historyhero", repository))
        .arg("properties.toy_type")
        .arg("plush")
        .arg("--message")
        .arg("set toy_type")
        .assert()
        .success();

    // View commit history
    nap_cmd()
        .arg("history")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("nap://{}/character/historyhero", repository))
        .arg("--limit")
        .arg("10")
        .assert()
        .success();
}

/// `create` and `set` commit automatically, but users can also edit a
/// working-tree manifest and use the explicit `commit`/`revert` commands.
/// Keep that lower-level workflow covered independently.
#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_explicit_commit_and_revert() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-explicit-commit");

    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(tmp.path())
        .args(["--provider", "local"])
        .assert()
        .success();
    nap_cmd()
        .args(["init", "--base-dir"])
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();
    nap_cmd()
        .args(["create", "--base-dir"])
        .arg(tmp.path())
        .args([
            "--repository",
            &repository,
            "character",
            "reverthero",
            "--name",
            "Revert Hero",
        ])
        .assert()
        .success();

    let manifest = tmp
        .path()
        .join(&repository)
        .join("character")
        .join("reverthero.yaml");
    let original = fs::read_to_string(&manifest).expect("read entity manifest");
    fs::write(
        &manifest,
        original.replace("Revert Hero", "Changed Revert Hero"),
    )
    .expect("update entity manifest");

    nap_cmd()
        .args(["commit", "--base-dir"])
        .arg(tmp.path())
        .arg(&repository)
        .args(["--message", "change revert hero"])
        .assert()
        .success();

    let commit_output = nap_cmd()
        .args(["--local", "head-hash", "--base-dir"])
        .arg(tmp.path())
        .arg(&repository)
        .output()
        .expect("run head-hash");
    assert!(
        commit_output.status.success(),
        "head-hash failed: {commit_output:?}"
    );
    let commit = String::from_utf8(commit_output.stdout)
        .expect("head-hash is utf-8")
        .trim()
        .to_string();
    assert!(!commit.is_empty(), "head-hash must return a commit hash");

    nap_cmd()
        .args(["revert", "--base-dir"])
        .arg(tmp.path())
        .arg(&repository)
        .args(["--commit", &commit])
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(manifest).expect("read reverted entity manifest"),
        original,
        "revert must restore the prior manifest contents"
    );
}

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_branch_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-branch");

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
        .arg(&repository)
        .assert()
        .success();

    // Create a branch
    nap_cmd()
        .arg("branch")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("feature-branch")
        .assert()
        .success();

    // List branches
    nap_cmd()
        .arg("branch")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success()
        .stdout(predicate::str::contains("feature-branch"));

    // Switch to the branch
    nap_cmd()
        .arg("switch")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("feature-branch")
        .assert()
        .success();
}

#[cfg(feature = "local-e2e")]
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
        .stdout(predicate::str::contains("provider_type"));

    // Run doctor
    nap_cmd()
        .arg("doctor")
        .arg("--base-dir")
        .arg(tmp.path())
        .assert()
        .success();
}

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_remote_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-remote");

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
        .arg(&repository)
        .assert()
        .success();

    // Add a remote
    nap_cmd()
        .arg("remote")
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("origin")
        .arg(format!("lore://localhost:41337/{}", repository))
        .assert()
        .success();

    // List remotes
    nap_cmd()
        .arg("remote")
        .arg("ls")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success()
        .stdout(predicate::str::contains("origin"));

    // Remotes are NAP worktree metadata.  Verify removal as well, rather
    // than only proving that the add path did not error.
    nap_cmd()
        .arg("remote")
        .arg("rm")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("origin")
        .assert()
        .success();

    nap_cmd()
        .arg("remote")
        .arg("ls")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success()
        .stdout(predicate::str::contains("origin").not());
}

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_sync_operations() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-sync");

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
        .arg(&repository)
        .assert()
        .success();

    // Add remote
    nap_cmd()
        .arg("remote")
        .arg("add")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("origin")
        .arg(format!("lore://localhost:41337/{}", repository))
        .assert()
        .success();

    // `publish` is the default-origin convenience command; `push` exposes
    // the explicit remote/branch variant.  Exercise both paths.
    nap_cmd()
        .arg("publish")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();

    nap_cmd()
        .arg("push")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .arg("--branch")
        .arg("main")
        .assert()
        .success();

    // Sync (pull)
    nap_cmd()
        .arg("sync")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(&repository)
        .assert()
        .success();
}

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_content_hash() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    // Create a test file
    let test_file = tmp.path().join("test.txt");
    fs::write(&test_file, "test content").expect("Failed to write test file");

    // Compute content hash
    nap_cmd()
        .arg("content-hash")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("blake3:"));
}

#[cfg(feature = "local-e2e")]
fn run_lore(repo_path: &Path, args: &[&str]) {
    let status = std::process::Command::new("lore")
        .args(args)
        .current_dir(repo_path)
        .status()
        .expect("failed to execute lore command");
    assert!(
        status.success(),
        "lore command failed: lore {}",
        args.join(" ")
    );
}

#[cfg(feature = "local-e2e")]
#[test]
fn test_local_lore_resolve_provenance_and_include_blobs() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let repository = unique_universe_name("test-provenance");

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
        .arg(&repository)
        .assert()
        .success();

    nap_cmd()
        .arg("create")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg("--repository")
        .arg(&repository)
        .arg("character")
        .arg("hero")
        .arg("--name")
        .arg("Provenance Hero")
        .arg("--author")
        .arg("integration-test")
        .assert()
        .success();

    let repo_path = tmp.path().join(&repository);
    let prompt_path = tmp.path().join("prompt.txt");
    fs::write(&prompt_path, "Readable prompt from Lore metadata.")
        .expect("failed to write prompt fixture");
    let prompt_path_string = prompt_path.to_string_lossy().into_owned();

    run_lore(
        &repo_path,
        &["stage", "character/hero.yaml", "--non-interactive"],
    );
    run_lore(
        &repo_path,
        &[
            "file",
            "metadata",
            "set",
            "character/hero.yaml",
            "nap.provenance.kind",
            "generation",
            "nap.provenance.model",
            "gpt-live",
            "--non-interactive",
        ],
    );
    run_lore(
        &repo_path,
        &[
            "file",
            "metadata",
            "set",
            "--binary",
            "character/hero.yaml",
            "nap.provenance.prompt.address",
            &prompt_path_string,
            "--non-interactive",
        ],
    );
    run_lore(
        &repo_path,
        &[
            "commit",
            "provenance metadata",
            "--identity",
            "integration-test",
            "--non-interactive",
        ],
    );

    let output = nap_cmd()
        .arg("resolve")
        .arg("--base-dir")
        .arg(tmp.path())
        .arg(format!("{repository}/character/hero"))
        .arg("--provenance")
        .arg("--include-blobs")
        .arg("-f")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).expect("resolve output is utf8");
    let envelope_start = stdout
        .find("{\n  \"manifest\"")
        .or_else(|| stdout.find("{\"manifest\""))
        .expect("resolve output contains a JSON envelope");
    let value: serde_json::Value =
        serde_json::from_str(&stdout[envelope_start..]).expect("valid JSON envelope output");
    assert_eq!(value["manifest"]["name"], "Provenance Hero");
    assert_eq!(
        value["provenance"]["files"][0]["path"],
        "character/hero.yaml"
    );
    assert_eq!(
        value["provenance"]["files"][0]["provenance"]["nap.provenance.kind"],
        "generation"
    );
    assert_eq!(
        value["provenance"]["files"][0]["provenance"]["nap.provenance.model"],
        "gpt-live"
    );
    assert_eq!(
        value["provenance"]["files"][0]["blobs"]["prompt"]["content"],
        "Readable prompt from Lore metadata."
    );
    assert_eq!(
        value["provenance"]["files"][0]["blobs"]["prompt"]["truncated"],
        false
    );
}
