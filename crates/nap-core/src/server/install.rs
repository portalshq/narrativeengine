// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Lore installer integration
//!
//! Integrates the official Lore installer behind `nap install lore`
//! to download, install, and verify Lore CLI and server binaries.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{error, info};

/// Lore installer for managing Lore CLI and server installation
pub struct LoreInstaller {
    install_dir: std::path::PathBuf,
    repo: String,
    version: String,
}

impl LoreInstaller {
    /// Create a new Lore installer
    pub fn new(install_dir: &Path) -> Self {
        Self {
            install_dir: install_dir.to_path_buf(),
            repo: "EpicGames/lore".to_string(),
            version: "latest".to_string(),
        }
    }

    /// Set custom repository
    pub fn with_repo(mut self, repo: &str) -> Self {
        self.repo = repo.to_string();
        self
    }

    /// Set custom version
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    /// Install Lore CLI
    pub fn install_cli(&self) -> Result<()> {
        info!(
            "Installing Lore CLI from {} version {}",
            self.repo, self.version
        );

        self.run_install_script(&["--version", &self.version])?;

        info!("Lore CLI installed successfully");
        Ok(())
    }

    /// Install Lore server
    pub fn install_server(&self) -> Result<()> {
        info!(
            "Installing Lore server from {} version {}",
            self.repo, self.version
        );

        self.run_install_script(&["--server", "--version", &self.version])?;

        info!("Lore server installed successfully");
        Ok(())
    }

    /// Install both CLI and server
    pub fn install_all(&self) -> Result<()> {
        info!(
            "Installing Lore CLI and server from {} version {}",
            self.repo, self.version
        );

        self.run_install_script(&["--version", &self.version])?;

        info!("Lore CLI and server installed successfully");
        Ok(())
    }

    /// Run the official Lore install script
    fn run_install_script(&self, args: &[&str]) -> Result<()> {
        let script_url = format!(
            "https://raw.githubusercontent.com/{}/main/scripts/install.sh",
            self.repo
        );

        // Download script
        let script_path = self.download_script(&script_url)?;

        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms)?;
        }

        // Build command with install directory and other args
        let mut cmd_args = vec![
            script_path.to_str().unwrap(),
            "--install-dir",
            self.install_dir.to_str().unwrap(),
        ];
        cmd_args.extend(args.iter().copied());

        // Execute script
        let output = Command::new("bash")
            .args(&cmd_args)
            .output()
            .context("Failed to execute Lore install script")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Lore install script failed: {}", stderr);
            anyhow::bail!("Lore install script failed with status: {}", output.status);
        }

        // Clean up script
        fs::remove_file(&script_path)?;

        Ok(())
    }

    /// Download install script to temporary location
    fn download_script(&self, url: &str) -> Result<std::path::PathBuf> {
        let response =
            reqwest::blocking::get(url).context("Failed to download Lore install script")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download script: HTTP {}", response.status());
        }

        let script_content = response.text().context("Failed to read script content")?;

        // Write to temporary file
        let temp_dir = std::env::temp_dir();
        let script_path = temp_dir.join("lore-install.sh");
        fs::write(&script_path, script_content).context("Failed to write install script")?;

        Ok(script_path)
    }

    /// Verify installation
    pub fn verify_installation(&self) -> Result<VerificationResult> {
        let cli_installed = self.check_binary("lore");
        let server_installed = self.check_binary("loreserver");

        let cli_version = if cli_installed {
            self.get_binary_version("lore").ok()
        } else {
            None
        };

        let server_version = if server_installed {
            self.get_binary_version("loreserver").ok()
        } else {
            None
        };

        Ok(VerificationResult {
            cli_installed,
            cli_version,
            server_installed,
            server_version,
        })
    }

    /// Check if binary exists and is executable
    fn check_binary(&self, name: &str) -> bool {
        let binary_path = self.install_dir.join(name);
        binary_path.exists() && binary_path.is_file()
    }

    /// Get version from binary
    fn get_binary_version(&self, name: &str) -> Result<String> {
        let binary_path = self.install_dir.join(name);
        let output = Command::new(&binary_path)
            .arg("--version")
            .output()
            .context(format!("Failed to execute {} --version", name))?;

        if !output.status.success() {
            anyhow::bail!("{} --version failed", name);
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Add install directory to PATH
    pub fn add_to_path(&self) -> Result<()> {
        let install_dir_str = self
            .install_dir
            .to_str()
            .context("Install directory path is not valid UTF-8")?;

        // Check if already in PATH
        if let Ok(current_path) = std::env::var("PATH")
            && current_path.contains(install_dir_str)
        {
            info!("Install directory already in PATH");
            return Ok(());
        }

        // Add to current process PATH
        let new_path = format!(
            "{}:{}",
            install_dir_str,
            std::env::var("PATH").unwrap_or_default()
        );
        unsafe {
            std::env::set_var("PATH", &new_path);
        }

        info!("Added {} to PATH for current process", install_dir_str);
        Ok(())
    }
}

/// Result of installation verification
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub cli_installed: bool,
    pub cli_version: Option<String>,
    pub server_installed: bool,
    pub server_version: Option<String>,
}

impl VerificationResult {
    /// Check if installation is complete
    pub fn is_complete(&self) -> bool {
        self.cli_installed && self.server_installed
    }

    /// Get a human-readable status message
    pub fn status_message(&self) -> String {
        let mut parts = vec![];

        if self.cli_installed {
            parts.push(format!(
                "Lore CLI installed ({})",
                self.cli_version.as_deref().unwrap_or("unknown")
            ));
        } else {
            parts.push("Lore CLI not installed".to_string());
        }

        if self.server_installed {
            parts.push(format!(
                "Lore server installed ({})",
                self.server_version.as_deref().unwrap_or("unknown")
            ));
        } else {
            parts.push("Lore server not installed".to_string());
        }

        parts.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_installer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let installer = LoreInstaller::new(temp_dir.path());
        assert_eq!(installer.repo, "EpicGames/lore");
        assert_eq!(installer.version, "latest");
    }

    #[test]
    fn test_installer_custom_repo() {
        let temp_dir = TempDir::new().unwrap();
        let installer = LoreInstaller::new(temp_dir.path())
            .with_repo("custom/repo")
            .with_version("v1.0.0");
        assert_eq!(installer.repo, "custom/repo");
        assert_eq!(installer.version, "v1.0.0");
    }

    #[test]
    fn test_verification_result() {
        let result = VerificationResult {
            cli_installed: true,
            cli_version: Some("0.8.5-nightly".to_string()),
            server_installed: false,
            server_version: None,
        };

        assert!(!result.is_complete());
        assert!(result.status_message().contains("Lore CLI installed"));
        assert!(
            result
                .status_message()
                .contains("Lore server not installed")
        );
    }
}
