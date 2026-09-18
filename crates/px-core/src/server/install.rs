// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Lore installer integration
//!
//! Integrates the official Lore installer behind `px install lore`
//! to download, install, and verify Lore CLI and server binaries.

use crate::server::error_ids;
use crate::server::{
    PINNED_LORE_ARTIFACT_MANIFEST_SHA256, PINNED_LORE_INSTALLER_PS1_SHA256,
    PINNED_LORE_INSTALLER_SHA256, PINNED_LORE_REPOSITORY, PINNED_LORE_VERSION,
};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::Command;
use tracing::{error, info};
use which;

/// Lore installer for managing Lore CLI and server installation
pub struct LoreInstaller {
    install_dir: Option<std::path::PathBuf>,
    repo: String,
    version: String,
    installer_sha256: String,
    manifest_sha256: String,
}

impl LoreInstaller {
    /// Create a new Lore installer
    pub fn new(install_dir: Option<std::path::PathBuf>) -> Self {
        Self {
            install_dir,
            repo: PINNED_LORE_REPOSITORY.to_string(),
            version: PINNED_LORE_VERSION.to_string(),
            installer_sha256: PINNED_LORE_INSTALLER_SHA256.to_string(),
            manifest_sha256: PINNED_LORE_ARTIFACT_MANIFEST_SHA256.to_string(),
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

    /// Set the expected checksum when deliberately selecting another release.
    pub fn with_installer_sha256(mut self, installer_sha256: &str) -> Self {
        self.installer_sha256 = installer_sha256.to_string();
        self
    }

    pub fn with_manifest_sha256(mut self, manifest_sha256: &str) -> Self {
        self.manifest_sha256 = manifest_sha256.to_string();
        self
    }

    /// Return the version with a `v` prefix for GitHub release tag lookups.
    ///
    /// GitHub releases use tags like `v0.8.4`, but [`PINNED_LORE_VERSION`]
    /// and `lore --version` report `0.8.4` (no prefix). The install script
    /// resolves releases by tag, so we must add the prefix here.
    fn tag_version(&self) -> String {
        if self.version.starts_with('v') {
            self.version.clone()
        } else {
            format!("v{}", self.version)
        }
    }

    /// Install Lore CLI (only if not already installed with correct version)
    pub fn install_cli(&self) -> Result<()> {
        // Check if already installed with correct version
        if let Ok(verification) = self.verify_installation()
            && verification.cli_installed
            && let Some(installed_version) = &verification.cli_version
        {
            // Strip build metadata for comparison (e.g., "0.8.4+283" -> "0.8.4")
            let installed_version_clean = installed_version
                .split('+')
                .next()
                .unwrap_or(installed_version);
            if installed_version_clean == self.version {
                info!(
                    "Lore CLI already installed with correct version {}",
                    installed_version
                );
                return Ok(());
            }
            info!(
                "Lore CLI installed but version mismatch: installed {}, required {}",
                installed_version, self.version
            );
        }

        info!(
            "Installing Lore CLI from {} version {}",
            self.repo, self.version
        );

        self.run_installer(false)?;

        info!("Lore CLI installed successfully");
        Ok(())
    }

    /// Install Lore server (only if not already installed with correct version)
    pub fn install_server(&self) -> Result<()> {
        // Check if already installed with correct version
        if let Ok(verification) = self.verify_installation()
            && verification.server_installed
            && let Some(installed_version) = &verification.server_version
        {
            // Strip build metadata for comparison (e.g., "0.8.4+283" -> "0.8.4")
            let installed_version_clean = installed_version
                .split('+')
                .next()
                .unwrap_or(installed_version);
            if installed_version_clean == self.version {
                info!(
                    "Lore server already installed with correct version {}",
                    installed_version
                );
                return Ok(());
            }
            info!(
                "Lore server installed but version mismatch: installed {}, required {}",
                installed_version, self.version
            );
        }

        info!(
            "Installing Lore server from {} version {}",
            self.repo, self.version
        );

        self.run_installer(true)?;

        info!("Lore server installed successfully");
        Ok(())
    }

    /// Install both CLI and server (only if not already installed with correct versions)
    pub fn install_all(&self) -> Result<()> {
        info!(
            "Checking Lore installation status for version {}",
            self.version
        );

        // The Lore install script installs one binary at a time:
        //   no flags  → lore CLI only
        //   --server  → loreserver only
        // Run it twice to get both.
        self.install_cli()?;
        self.install_server()?;

        info!("Lore CLI and server installation verified");
        Ok(())
    }

    /// Run the official Lore installer for this platform: `install.sh`
    /// via bash on unix, `install.ps1` via PowerShell on Windows.
    fn run_installer(&self, server_only: bool) -> Result<()> {
        if cfg!(windows) {
            return self.run_install_ps1(server_only);
        }
        let tag = self.tag_version();
        if server_only {
            self.run_install_sh(&["--server", "--version", &tag])
        } else {
            self.run_install_sh(&["--version", &tag])
        }
    }

    /// Run the official Lore install script
    fn run_install_sh(&self, args: &[&str]) -> Result<()> {
        let script_url = format!(
            "https://raw.githubusercontent.com/{}/{}/scripts/install.sh",
            self.repo,
            self.tag_version(),
        );

        // Download script
        let script_path = self.download_script(&script_url, &self.installer_sha256, "sh")?;

        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o700);
            fs::set_permissions(&script_path, perms)?;
        }

        // Build command with install directory and other args
        let script_arg = script_path
            .to_str()
            .context("Lore installer temporary path is not valid UTF-8")?;
        let mut cmd_args = vec![script_arg];
        if let Some(dir) = &self.install_dir {
            cmd_args.push("--install-dir");
            cmd_args.push(
                dir.to_str()
                    .context("Lore installation directory is not valid UTF-8")?,
            );
        }
        // NOTE: --repo is intentionally not passed. The pinned v0.8.4-portals.9
        // installer rejects unknown arguments; its default REPO=portalshq/lore
        // already matches PINNED_LORE_REPOSITORY. Custom with_repo() installs
        // are deferred until Lore ships --repo support (follow-up .6).
        cmd_args.push("--manifest-sha256");
        cmd_args.push(&self.manifest_sha256);
        cmd_args.extend(args.iter().copied());

        // Execute script
        let output_result = Command::new("bash").args(&cmd_args).output();

        // Remove the downloaded program even when process creation fails, and
        // before examining the exit status, so executable material is never
        // left in a shared temp directory.
        fs::remove_file(&script_path).context("Failed to remove Lore installer script")?;
        let output = output_result.context(format!(
            "[{}] Failed to execute Lore install script",
            error_ids::ERR_LORE_INSTALL_FAILED
        ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(
                "[{}] Lore install script failed: {}",
                error_ids::ERR_LORE_INSTALL_FAILED,
                stderr
            );
            anyhow::bail!(
                "[{}] Lore install script failed with status: {}",
                error_ids::ERR_LORE_INSTALL_FAILED,
                output.status
            );
        }

        Ok(())
    }

    /// Run the official Lore `install.ps1` via inbox PowerShell.
    ///
    /// Mirrors `run_install_sh`: same pinned tag, same signed-manifest
    /// digest (already `sha256:`-prefixed, as the script requires), same
    /// CLI-vs-server selection — only the script language and its flags
    /// differ (`-Version` / `-ManifestSha256` / `-Server`).
    fn run_install_ps1(&self, server_only: bool) -> Result<()> {
        let script_url = format!(
            "https://raw.githubusercontent.com/{}/{}/scripts/install.ps1",
            self.repo,
            self.tag_version(),
        );

        let script_path =
            self.download_script(&script_url, PINNED_LORE_INSTALLER_PS1_SHA256, "ps1")?;
        let script_arg = script_path
            .to_str()
            .context("Lore installer temporary path is not valid UTF-8")?;
        // Without an explicit dir the script defaults to %USERPROFILE%\bin
        // and persists it to the User PATH; px additionally prepends it to
        // this process's PATH below so post-install verification resolves.
        let install_dir = self
            .install_dir
            .clone()
            .or_else(default_windows_install_dir)
            .context(format!(
                "[{}] Cannot determine Lore install directory on Windows",
                error_ids::ERR_LORE_INSTALL_FAILED
            ))?;
        let install_dir_str = install_dir
            .to_str()
            .context("Lore installation directory is not valid UTF-8")?;

        let mut cmd = Command::new(powershell_binary());
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            script_arg,
            "-Version",
            &self.tag_version(),
            "-ManifestSha256",
            &self.manifest_sha256,
            "-InstallDir",
            install_dir_str,
        ]);
        if server_only {
            cmd.arg("-Server");
        }
        let output_result = cmd.output();

        // Remove the downloaded program even when process creation fails, and
        // before examining the exit status, so executable material is never
        // left in a shared temp directory.
        fs::remove_file(&script_path).context("Failed to remove Lore installer script")?;
        let output = output_result.context(format!(
            "[{}] Failed to execute Lore install script",
            error_ids::ERR_LORE_INSTALL_FAILED
        ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(
                "[{}] Lore install script failed: {}",
                error_ids::ERR_LORE_INSTALL_FAILED,
                stderr
            );
            anyhow::bail!(
                "[{}] Lore install script failed with status: {}",
                error_ids::ERR_LORE_INSTALL_FAILED,
                output.status
            );
        }

        prepend_to_process_path(&install_dir)?;

        Ok(())
    }

    /// Download install script to temporary location
    ///
    /// Runs on a dedicated OS thread: `reqwest::blocking` owns a Tokio
    /// runtime that must be created *and dropped* outside any async
    /// context, but installers run inside `Runtime::block_on`
    /// (`px configure local`, `px doctor`, …). Dropping it on a runtime
    /// thread panics with "Cannot drop a runtime…".
    fn download_script(
        &self,
        url: &str,
        expected_sha256: &str,
        extension: &str,
    ) -> Result<std::path::PathBuf> {
        let url = url.to_string();
        // Consume the whole response on the spawned thread and move only
        // owned data back: every `reqwest::blocking` call (including
        // `Response::bytes`) builds and drops a throwaway runtime in debug
        // builds, which panics on a Tokio thread.
        let (status, script_content) = std::thread::Builder::new()
            .name("px-lore-download".to_string())
            .spawn(move || {
                let response = reqwest::blocking::get(&url).map_err(|e| {
                    anyhow::anyhow!(
                        "[{}] Failed to download Lore install script: {e}",
                        error_ids::ERR_LORE_DOWNLOAD_FAILED
                    )
                })?;
                let status = response.status();
                let bytes = response.bytes().map_err(|e| {
                    anyhow::anyhow!(
                        "[{}] Failed to read installer bytes: {e}",
                        error_ids::ERR_LORE_DOWNLOAD_FAILED
                    )
                })?;
                Ok::<_, anyhow::Error>((status, bytes))
            })
            .context(format!(
                "[{}] Failed to spawn Lore download thread",
                error_ids::ERR_LORE_DOWNLOAD_FAILED
            ))?
            .join()
            .map_err(|_| {
                anyhow::anyhow!(
                    "[{}] Lore download thread panicked",
                    error_ids::ERR_LORE_DOWNLOAD_FAILED
                )
            })??;

        if !status.is_success() {
            anyhow::bail!(
                "[{}] Failed to download script: HTTP {}",
                error_ids::ERR_LORE_DOWNLOAD_FAILED,
                status
            );
        }

        let actual_sha256 = hex::encode(Sha256::digest(&script_content));
        if actual_sha256 != expected_sha256 {
            anyhow::bail!(
                "[{}] Lore installer checksum mismatch for {} {}: expected {}, got {}",
                error_ids::ERR_LORE_DOWNLOAD_FAILED,
                self.repo,
                self.tag_version(),
                expected_sha256,
                actual_sha256,
            );
        }

        // create_new prevents symlink-following and pre-creation attacks. PID
        // plus a cryptographically random nonce avoids cross-process and
        // concurrent-use collisions without trusting a predictable filename.
        let nonce = rand::random::<u64>();
        let script_path = std::env::temp_dir().join(format!(
            "px-lore-install-{}-{nonce:016x}.{extension}",
            std::process::id(),
        ));
        let mut script_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&script_path)
            .context(format!(
                "[{}] Failed to create installer safely",
                error_ids::ERR_LORE_DOWNLOAD_FAILED
            ))?;
        script_file.write_all(&script_content).context(format!(
            "[{}] Failed to write install script",
            error_ids::ERR_LORE_DOWNLOAD_FAILED
        ))?;

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
        if let Some(dir) = &self.install_dir {
            let binary_path = dir.join(name);
            binary_path.exists() && binary_path.is_file()
        } else {
            // Check system PATH
            which::which(name).is_ok()
        }
    }

    /// Get version from binary
    ///
    /// Handles both output formats:
    /// - `"0.8.4+283"` (just the version)
    /// - `"lore 0.8.4+283"` (program name prefix, common on macOS)
    ///
    /// Returns the clean version string (e.g. `"0.8.4+283"`).
    fn get_binary_version(&self, name: &str) -> Result<String> {
        let binary_path = if let Some(dir) = &self.install_dir {
            dir.join(name).to_str().unwrap().to_string()
        } else {
            name.to_string() // Rely on PATH
        };

        let output = Command::new(&binary_path)
            .arg("--version")
            .output()
            .context(format!("Failed to execute {} --version", binary_path))?;

        if !output.status.success() {
            anyhow::bail!("{} --version failed", name);
        }

        let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(parse_version_output(&raw))
    }

    /// Add install directory to PATH
    pub fn add_to_path(&self) -> Result<()> {
        let install_dir = if let Some(dir) = &self.install_dir {
            dir
        } else {
            return Ok(()); // Already in PATH or system default
        };

        let install_dir_str = install_dir
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

/// Shell used to run `install.ps1`: prefer PowerShell 7 when installed,
/// fall back to inbox Windows PowerShell 5.1 (the script supports both).
fn powershell_binary() -> String {
    if which::which("pwsh").is_ok() {
        "pwsh".to_string()
    } else {
        "powershell".to_string()
    }
}

/// Default Lore install directory on Windows, matching `install.ps1`
/// (`%USERPROFILE%\bin`). Unix keeps `None` (the shell script default,
/// already on PATH).
#[cfg(windows)]
fn default_windows_install_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("USERPROFILE").map(|p| std::path::PathBuf::from(p).join("bin"))
}

#[cfg(not(windows))]
fn default_windows_install_dir() -> Option<std::path::PathBuf> {
    None
}

/// Prepend a directory to this process's PATH using the platform separator.
///
/// `install.ps1` persists its directory to the User PATH registry key, but
/// that never reaches the running px process, so post-install verification
/// (`which lore`) needs the prepend here.
fn prepend_to_process_path(dir: &std::path::Path) -> Result<()> {
    let dir_str = dir
        .to_str()
        .context("Install directory path is not valid UTF-8")?;
    let sep = if cfg!(windows) { ';' } else { ':' };
    if let Ok(current) = std::env::var("PATH")
        && current.split(sep).any(|p| p == dir_str)
    {
        return Ok(());
    }
    let new_path = format!(
        "{dir_str}{sep}{}",
        std::env::var("PATH").unwrap_or_default()
    );
    unsafe {
        std::env::set_var("PATH", &new_path);
    }
    info!("Added {} to PATH for current process", dir_str);
    Ok(())
}

/// Parse the output of `lore --version` (or `loreserver --version`).
///
/// Handles:
/// - `"0.8.4+283"` -> `"0.8.4+283"`
/// - `"lore 0.8.4+283"` -> `"0.8.4+283"`
/// - `"my-tool 1.2.3"` -> `"1.2.3"`
pub fn parse_version_output(raw: &str) -> String {
    let raw = raw.trim();
    if let Some(pos) = raw.rfind(' ') {
        // Take the last token after the final space
        raw[pos + 1..].to_string()
    } else {
        raw.to_string()
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
        let installer = LoreInstaller::new(Some(temp_dir.path().to_path_buf()));
        assert_eq!(installer.repo, PINNED_LORE_REPOSITORY);
        assert_eq!(installer.version, PINNED_LORE_VERSION);
        assert_eq!(installer.installer_sha256, PINNED_LORE_INSTALLER_SHA256);
        // Tag version must have the `v` prefix for GitHub release lookups
        assert_eq!(installer.tag_version(), format!("v{}", PINNED_LORE_VERSION));
    }

    #[test]
    fn test_tag_version_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let installer = LoreInstaller::new(Some(temp_dir.path().to_path_buf()));
        assert_eq!(installer.tag_version(), "v0.8.4-portals.9");

        // Already prefixed — should not double-prefix
        let installer2 =
            LoreInstaller::new(Some(temp_dir.path().to_path_buf())).with_version("v1.0.0");
        assert_eq!(installer2.tag_version(), "v1.0.0");
    }

    #[test]
    fn test_parse_version_output() {
        assert_eq!(parse_version_output("0.8.4+283"), "0.8.4+283");
        assert_eq!(parse_version_output("lore 0.8.4+283"), "0.8.4+283");
        assert_eq!(parse_version_output("loreserver 0.8.4+283"), "0.8.4+283");
        assert_eq!(parse_version_output("my-tool 1.2.3"), "1.2.3");
        assert_eq!(parse_version_output("some-tool"), "some-tool");
    }

    #[test]
    fn test_installer_custom_repo() {
        let temp_dir = TempDir::new().unwrap();
        let installer = LoreInstaller::new(Some(temp_dir.path().to_path_buf()))
            .with_repo("custom/repo")
            .with_version("v1.0.0");
        assert_eq!(installer.repo, "custom/repo");
        assert_eq!(installer.version, "v1.0.0");
    }

    #[test]
    fn test_download_script_inside_tokio_runtime() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        // `px configure local` / `px doctor` run provider init inside
        // `Runtime::block_on`. The installer download must survive that
        // context instead of panicking with "Cannot drop a runtime in a
        // context where blocking is not allowed".
        let body = b"fake-installer-script";
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(header.as_bytes()).unwrap();
            stream.write_all(body).unwrap();
        });

        let sha = hex::encode(Sha256::digest(body));
        let installer = LoreInstaller::new(None).with_installer_sha256(&sha);
        let url = format!("http://{addr}/install.sh");

        let rt = tokio::runtime::Runtime::new().unwrap();
        let path = rt
            .block_on(async { installer.download_script(&url, &sha, "sh") })
            .expect("download inside a tokio runtime must not panic");
        let saved = std::fs::read(&path).unwrap();
        assert_eq!(saved, body);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_verification_result() {
        let result = VerificationResult {
            cli_installed: true,
            cli_version: Some("0.8.4".to_string()),
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
