// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Integration tests for Lore server version pinning.
//!
//! These tests enforce the contract that NAP requires an **exact** version
//! match (including the release-channel suffix) when the local lore server
//! is initialized.  A mismatched version — even one that shares the same
//! major.minor.patch — must be rejected.
//!
//! Run with:
//!   cargo test -p nap-core --test lore_version_integration

use nap_core::server::{LoreVersionInfo, PINNED_LORE_VERSION, check_lore_compatibility};
use semver::Version;

// ── Extensible test variable ────────────────────────────────────────────
//
// Change this single constant to update the pinned version across every
// integration test in this file.  The value **must** agree with the
// production constant `PINNED_LORE_VERSION` or the contract tests below
// will fail loudly.
const EXPECTED_PINNED_VERSION: &str = "0.8.5-nightly";

// ── Helpers ─────────────────────────────────────────────────────────────

/// Build a `LoreVersionInfo` from a raw version string and its semver parts.
fn make_version_info(raw: &str, major: u64, minor: u64, patch: u64) -> LoreVersionInfo {
    LoreVersionInfo {
        parsed: Version::new(major, minor, patch),
        raw: raw.to_string(),
    }
}

// ── Contract: constant value ────────────────────────────────────────────

/// The pinned version constant must be exactly the expected nightly string.
#[test]
fn pinned_version_constant_matches_expected() {
    assert_eq!(
        PINNED_LORE_VERSION, EXPECTED_PINNED_VERSION,
        "PINNED_LORE_VERSION has changed!  Update EXPECTED_PINNED_VERSION \
         in this test file if the change is intentional."
    );
}

// ── Contract: exact match passes ────────────────────────────────────────

/// A lore server that reports the exact pinned version must be accepted.
#[test]
fn exact_nightly_version_is_compatible() {
    let installed = make_version_info("0.8.5-nightly", 0, 8, 5);
    assert!(
        check_lore_compatibility(&installed).unwrap(),
        "Version '0.8.5-nightly' should be compatible with pinned version '{}'",
        PINNED_LORE_VERSION
    );
}

// ── Contract: mismatches are rejected ───────────────────────────────────

/// Bare version without channel suffix must be rejected.
#[test]
fn bare_version_without_suffix_is_incompatible() {
    let installed = make_version_info("0.8.5", 0, 8, 5);
    assert!(
        !check_lore_compatibility(&installed).unwrap(),
        "Version '0.8.5' (no suffix) must NOT be compatible with '{}'",
        PINNED_LORE_VERSION
    );
}

/// Same semver but different channel suffix must be rejected.
#[test]
fn stable_channel_is_incompatible_with_nightly_pin() {
    let installed = make_version_info("0.8.5-stable", 0, 8, 5);
    assert!(
        !check_lore_compatibility(&installed).unwrap(),
        "Version '0.8.5-stable' must NOT be compatible with '{}'",
        PINNED_LORE_VERSION
    );
}

/// Different major.minor.patch must be rejected.
#[test]
fn wrong_semver_is_incompatible() {
    let installed = make_version_info("0.9.0-nightly", 0, 9, 0);
    assert!(
        !check_lore_compatibility(&installed).unwrap(),
        "Version '0.9.0-nightly' must NOT be compatible with '{}'",
        PINNED_LORE_VERSION
    );
}

/// Older version must be rejected.
#[test]
fn older_version_is_incompatible() {
    let installed = make_version_info("0.7.0-nightly", 0, 7, 0);
    assert!(
        !check_lore_compatibility(&installed).unwrap(),
        "Version '0.7.0-nightly' must NOT be compatible with '{}'",
        PINNED_LORE_VERSION
    );
}

/// Release-candidate suffix must be rejected.
#[test]
fn release_candidate_is_incompatible_with_nightly_pin() {
    let installed = make_version_info("0.8.5-rc1", 0, 8, 5);
    assert!(
        !check_lore_compatibility(&installed).unwrap(),
        "Version '0.8.5-rc1' must NOT be compatible with '{}'",
        PINNED_LORE_VERSION
    );
}

/// Pre-release suffix must be rejected.
#[test]
fn pre_release_suffix_is_incompatible_with_nightly_pin() {
    let installed = make_version_info("0.8.5-pre", 0, 8, 5);
    assert!(
        !check_lore_compatibility(&installed).unwrap(),
        "Version '0.8.5-pre' must NOT be compatible with '{}'",
        PINNED_LORE_VERSION
    );
}

/// Higher version must be rejected — compatibility is exact, not >=.
#[test]
fn newer_version_is_incompatible() {
    let installed = make_version_info("0.9.0-nightly", 0, 9, 0);
    assert!(
        !check_lore_compatibility(&installed).unwrap(),
        "Version '0.9.0-nightly' must NOT be compatible with '{}'",
        PINNED_LORE_VERSION
    );
}

// ── Contract: LoreVersionInfo round-trip ────────────────────────────────

/// Verify that `LoreVersionInfo` preserves both raw and parsed representations.
#[test]
fn version_info_preserves_raw_and_parsed() {
    let info = make_version_info("0.8.5-nightly", 0, 8, 5);
    assert_eq!(info.raw, "0.8.5-nightly");
    assert_eq!(info.parsed, Version::new(0, 8, 5));
}

/// Verify Display implementation shows the raw version.
#[test]
fn version_info_display_shows_raw() {
    let info = make_version_info("0.8.5-nightly", 0, 8, 5);
    assert_eq!(format!("{}", info), "0.8.5-nightly");
}
