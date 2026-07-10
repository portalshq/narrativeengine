// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Lore version detection and compatibility checking
//!
//! Provides utilities to detect installed Lore CLI/server versions and
//! verify compatibility with the SDK's pinned version.

use anyhow::{Context, Result};
use std::process::Command;
use semver::Version;

/// Pinned Lore version that NAP SDK requires
pub const PINNED_LORE_VERSION: &str = "0.8.5-nightly";

/// Detect the installed Lore CLI version
pub fn detect_lore_version() -> Result<Version> {
    let output = Command::new("lore")
        .arg("--version")
        .output()
        .context(
            "Failed to execute 'lore --version'. \
             Lore CLI is not installed or not on PATH. \
             Install it with: nap install lore"
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
    parse_lore_version(&version_str)
}

/// Detect the installed Lore server version
pub fn detect_loreserver_version() -> Result<Version> {
    let output = Command::new("loreserver")
        .arg("--version")
        .output()
        .context(
            "Failed to execute 'loreserver --version'. \
             Lore server is not installed or not on PATH. \
             Install it with: nap install lore"
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
    parse_lore_version(&version_str)
}

/// Parse Lore version string into semver::Version
fn parse_lore_version(version_str: &str) -> Result<Version> {
    // Lore version format: "lore 0.8.5-nightly" or "loreserver 0.8.5-nightly"
    let version_part = version_str
        .split_whitespace()
        .nth(1)
        .context(format!(
            "Failed to parse Lore version string '{}'. \
             Expected format: 'lore <version>' (e.g., 'lore 0.8.5-nightly')",
            version_str.trim()
        ))?;

    // Handle nightly versions by stripping the suffix for comparison
    let version_for_semver = version_part.trim_end_matches("-nightly");
    
    Version::parse(version_for_semver)
        .context(format!(
            "Failed to parse '{}' as semver version. \
             Lore version string may be in an unexpected format.",
            version_for_semver
        ))
}

/// Check if installed Lore version is compatible with pinned version
pub fn check_lore_compatibility(installed_version: &Version) -> Result<bool> {
    let pinned = Version::parse(PINNED_LORE_VERSION.trim_end_matches("-nightly"))
        .context("Failed to parse pinned Lore version")?;

    // For now, require exact match on major.minor.patch
    // Nightly suffix is ignored for comparison
    let installed_clean = Version::new(
        installed_version.major,
        installed_version.minor,
        installed_version.patch,
    );
    
    let pinned_clean = Version::new(
        pinned.major,
        pinned.minor,
        pinned.patch,
    );

    Ok(installed_clean == pinned_clean)
}

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

/// Status of Lore installation
#[derive(Debug, Clone)]
pub struct LoreInstallationStatus {
    pub cli_installed: bool,
    pub cli_version: Option<Version>,
    pub cli_compatible: bool,
    pub server_installed: bool,
    pub server_version: Option<Version>,
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
                "Lore CLI version {:?} is incompatible with pinned version {}",
                self.cli_version, self.pinned_version
            ));
        }

        if !self.server_installed {
            messages.push("Lore server is not installed".to_string());
        } else if !self.server_compatible {
            messages.push(format!(
                "Lore server version {:?} is incompatible with pinned version {}",
                self.server_version, self.pinned_version
            ));
        }

        if messages.is_empty() {
            "Lore installation is compatible".to_string()
        } else {
            messages.join("; ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lore_version() {
        let version_str = "lore 0.8.5-nightly";
        let version = parse_lore_version(version_str).unwrap();
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 8);
        assert_eq!(version.patch, 5);
    }

    #[test]
    fn test_parse_loreserver_version() {
        let version_str = "loreserver 0.8.5-nightly";
        let version = parse_lore_version(version_str).unwrap();
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 8);
        assert_eq!(version.patch, 5);
    }

    #[test]
    fn test_compatibility_check() {
        let installed = Version::new(0, 8, 5);
        assert!(check_lore_compatibility(&installed).unwrap());

        let incompatible = Version::new(0, 7, 0);
        assert!(!check_lore_compatibility(&incompatible).unwrap());
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
            pinned_version: "0.8.5-nightly".to_string(),
        };
        
        let message = status.status_message();
        assert!(message.contains("Lore CLI is not installed"));
        assert!(message.contains("Lore server is not installed"));
    }
}
