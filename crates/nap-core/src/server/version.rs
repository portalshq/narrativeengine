// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Lore version detection and compatibility checking
//!
//! Provides utilities to detect installed Lore CLI/server versions and
//! verify compatibility with the SDK's pinned version.
//!
//! NAP requires an **exact match** of the full version string against
//! [`PINNED_LORE_VERSION`].

use anyhow::{Context, Result};
use semver::Version;
use std::process::Command;

/// Pinned Lore version that NAP SDK requires.
///
/// During initialization NAP verifies that the installed `lore` and
/// `loreserver` binaries report **exactly** this version string.
pub const PINNED_LORE_VERSION: &str = "0.8.4";

// ── Detected version info ───────────────────────────────────────────────

/// A detected Lore version with both parsed semver and raw string forms.
///
/// The `raw` field preserves the full version string reported by the CLI
/// so that compatibility checks can enforce an exact match — not just
/// major.minor.patch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoreVersionInfo {
    /// Parsed semver (e.g. `0.8.4`).  The nightly suffix is stripped here
    /// because `semver::Version` has no concept of release channels.
    pub parsed: Version,
    /// Raw version string exactly as reported by the binary
    /// (e.g. `"0.8.4"`).
    pub raw: String,
}

impl std::fmt::Display for LoreVersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

// ── Version detection ───────────────────────────────────────────────────

/// Extract the raw version token from CLI output.
///
/// Given `"lore 0.8.4\n"`, returns `"0.8.4"`.
fn extract_version_string(version_str: &str) -> Result<String> {
    let version_part = version_str.split_whitespace().nth(1).context(format!(
        "Failed to parse Lore version string '{}'. \
             Expected format: 'lore <version>' (e.g., 'lore 0.8.4')",
        version_str.trim()
    ))?;
    Ok(version_part.trim().to_string())
}

/// Detect the installed Lore CLI version
pub fn detect_lore_version() -> Result<LoreVersionInfo> {
    let output = Command::new("lore").arg("--version").output().context(
        "Failed to execute 'lore --version'. \
             Lore CLI is not installed or not on PATH. \
             Install it with: nap install lore",
    )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "lore --version exited with status: {}. stderr: {}",
            output.status,
            stderr.trim()
        );
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    let raw = extract_version_string(&version_str)?;
    let parsed = parse_lore_version(&version_str)?;
    Ok(LoreVersionInfo { parsed, raw })
}

/// Detect the installed Lore server version
pub fn detect_loreserver_version() -> Result<LoreVersionInfo> {
    let output = Command::new("loreserver")
        .arg("--version")
        .output()
        .context(
            "Failed to execute 'loreserver --version'. \
             Lore server is not installed or not on PATH. \
             Install it with: nap install lore",
        )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "loreserver --version exited with status: {}. stderr: {}",
            output.status,
            stderr.trim()
        );
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    let raw = extract_version_string(&version_str)?;
    let parsed = parse_lore_version(&version_str)?;
    Ok(LoreVersionInfo { parsed, raw })
}

// ── Version parsing ─────────────────────────────────────────────────────

/// Parse Lore version string into `semver::Version`.
///
/// Strips the nightly/release suffix before parsing because
/// `semver::Version` does not model release channels.  Use
/// [`extract_version_string`] when you need the full, unparsed token.
fn parse_lore_version(version_str: &str) -> Result<Version> {
    // Lore version format: "lore 0.8.4" or "loreserver 0.8.4"
    let version_part = version_str.split_whitespace().nth(1).context(format!(
        "Failed to parse Lore version string '{}'. \
             Expected format: 'lore <version>' (e.g., 'lore 0.8.4')",
        version_str.trim()
    ))?;

    // Handle nightly versions by stripping the suffix for semver parsing
    let version_for_semver = version_part.trim_end_matches("-nightly");

    Version::parse(version_for_semver).context(format!(
        "Failed to parse '{}' as semver version. \
             Lore version string may be in an unexpected format.",
        version_for_semver
    ))
}

// ── Compatibility gate ──────────────────────────────────────────────────

/// Check if the installed Lore version **exactly** matches the pinned version.
///
/// The comparison is a strict string equality of the raw version tokens,
/// so `"0.8.4"` matches but `"0.8.4-nightly"` or `"0.8.4-stable"` does not.
pub fn check_lore_compatibility(installed: &LoreVersionInfo) -> Result<bool> {
    Ok(installed.raw == PINNED_LORE_VERSION)
}

// ── Full installation verification ──────────────────────────────────────

/// Verify Lore installation and compatibility
pub fn verify_lore_installation() -> Result<LoreInstallationStatus> {
    let cli_version = match detect_lore_version() {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::warn!("Failed to detect Lore CLI version: {}", e);
            None
        }
    };

    let server_version = match detect_loreserver_version() {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::warn!("Failed to detect Lore server version: {}", e);
            None
        }
    };

    let cli_compatible = cli_version
        .as_ref()
        .map(|v| check_lore_compatibility(v).unwrap_or(false))
        .unwrap_or(false);

    let server_compatible = server_version
        .as_ref()
        .map(|v| check_lore_compatibility(v).unwrap_or(false))
        .unwrap_or(false);

    Ok(LoreInstallationStatus {
        cli_installed: cli_version.is_some(),
        cli_version,
        cli_compatible,
        server_installed: server_version.is_some(),
        server_version,
        server_compatible,
        pinned_version: PINNED_LORE_VERSION.to_string(),
    })
}

// ── Installation status ─────────────────────────────────────────────────

/// Status of Lore installation
#[derive(Debug, Clone)]
pub struct LoreInstallationStatus {
    pub cli_installed: bool,
    pub cli_version: Option<LoreVersionInfo>,
    pub cli_compatible: bool,
    pub server_installed: bool,
    pub server_version: Option<LoreVersionInfo>,
    pub server_compatible: bool,
    pub pinned_version: String,
}

impl LoreInstallationStatus {
    /// Check if installation is fully compatible
    pub fn is_fully_compatible(&self) -> bool {
        self.cli_installed && self.cli_compatible && self.server_installed && self.server_compatible
    }

    /// Get a human-readable status message
    pub fn status_message(&self) -> String {
        let mut messages = vec![];

        if !self.cli_installed {
            messages.push("Lore CLI is not installed".to_string());
        } else if !self.cli_compatible {
            messages.push(format!(
                "Lore CLI version '{}' is incompatible with required version '{}'",
                self.cli_version
                    .as_ref()
                    .map(|v| v.raw.as_str())
                    .unwrap_or("unknown"),
                self.pinned_version
            ));
        }

        if !self.server_installed {
            messages.push("Lore server is not installed".to_string());
        } else if !self.server_compatible {
            messages.push(format!(
                "Lore server version '{}' is incompatible with required version '{}'",
                self.server_version
                    .as_ref()
                    .map(|v| v.raw.as_str())
                    .unwrap_or("unknown"),
                self.pinned_version
            ));
        }

        if messages.is_empty() {
            "Lore installation is compatible".to_string()
        } else {
            messages.join("; ")
        }
    }
}

// ── Unit tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version_string() {
        assert_eq!(
            extract_version_string("lore 0.8.4").unwrap(),
            "0.8.4"
        );
        assert_eq!(
            extract_version_string("loreserver 0.8.4").unwrap(),
            "0.8.4"
        );
        assert_eq!(extract_version_string("lore 0.8.4\n").unwrap(), "0.8.4");
    }

    #[test]
    fn test_extract_version_string_failure() {
        // Single-word input has no second token → should fail
        assert!(extract_version_string("lore").is_err());
        // Empty string → should fail
        assert!(extract_version_string("").is_err());
    }

    #[test]
    fn test_parse_lore_version() {
        let version_str = "lore 0.8.4";
        let version = parse_lore_version(version_str).unwrap();
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 8);
        assert_eq!(version.patch, 4);
    }

    #[test]
    fn test_parse_lore_version_with_nightly_suffix() {
        // Nightly suffix is stripped for semver parsing
        let version_str = "lore 0.8.4-nightly";
        let version = parse_lore_version(version_str).unwrap();
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 8);
        assert_eq!(version.patch, 4);
    }

    #[test]
    fn test_parse_loreserver_version() {
        let version_str = "loreserver 0.8.4";
        let version = parse_lore_version(version_str).unwrap();
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 8);
        assert_eq!(version.patch, 4);
    }

    #[test]
    fn test_compatibility_exact_match() {
        let installed = LoreVersionInfo {
            parsed: Version::new(0, 8, 4),
            raw: "0.8.4".to_string(),
        };
        assert!(check_lore_compatibility(&installed).unwrap());
    }

    #[test]
    fn test_compatibility_rejects_nightly_suffix() {
        // "0.8.4-nightly" must NOT match pinned "0.8.4"
        let installed = LoreVersionInfo {
            parsed: Version::new(0, 8, 4),
            raw: "0.8.4-nightly".to_string(),
        };
        assert!(!check_lore_compatibility(&installed).unwrap());
    }

    #[test]
    fn test_compatibility_rejects_wrong_channel() {
        let installed = LoreVersionInfo {
            parsed: Version::new(0, 8, 4),
            raw: "0.8.4-stable".to_string(),
        };
        assert!(!check_lore_compatibility(&installed).unwrap());
    }

    #[test]
    fn test_compatibility_rejects_wrong_version() {
        let installed = LoreVersionInfo {
            parsed: Version::new(0, 7, 0),
            raw: "0.7.0".to_string(),
        };
        assert!(!check_lore_compatibility(&installed).unwrap());
    }

    #[test]
    fn test_installation_status_message() {
        let status = LoreInstallationStatus {
            cli_installed: false,
            cli_version: None,
            cli_compatible: false,
            server_installed: false,
            server_version: None,
            server_compatible: false,
            pinned_version: PINNED_LORE_VERSION.to_string(),
        };

        let message = status.status_message();
        assert!(message.contains("Lore CLI is not installed"));
        assert!(message.contains("Lore server is not installed"));
    }

    #[test]
    fn test_installation_status_message_incompatible() {
        let status = LoreInstallationStatus {
            cli_installed: true,
            cli_version: Some(LoreVersionInfo {
                parsed: Version::new(0, 8, 4),
                raw: "0.8.4-nightly".to_string(),
            }),
            cli_compatible: false,
            server_installed: true,
            server_version: Some(LoreVersionInfo {
                parsed: Version::new(0, 8, 4),
                raw: "0.8.4-nightly".to_string(),
            }),
            server_compatible: false,
            pinned_version: PINNED_LORE_VERSION.to_string(),
        };

        let message = status.status_message();
        assert!(message.contains("'0.8.4-nightly'"));
        assert!(message.contains("'0.8.4'"));
        assert!(!status.is_fully_compatible());
    }

    #[test]
    fn test_pinned_version_constant() {
        // This test documents the contract: the pinned version must be
        // "0.8.4".  If you intentionally change it, update this test and
        // the integration test as well.
        assert_eq!(PINNED_LORE_VERSION, "0.8.4");
    }
}
