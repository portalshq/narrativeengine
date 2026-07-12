// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Server manager for Lore repository backend
//!
//! Provides complete lifecycle management for the Local provider:
//! - ensureInstalled()
//! - ensureConfigured()
//! - ensureRunning()
//! - start()
//! - stop()
//! - restart()
//! - status()
//! - healthCheck()
//! - upgrade()

use anyhow::{Context, Result};
use std::path::Path;
use std::time::Duration;
use tracing::{error, info, warn};

use super::{
    cert::generate_certificates, config::generate_local_config, lock::ProcessLock,
    process::LoreProcessManager, version::verify_lore_installation,
};

use crate::server::error_ids;
use crate::LoreInstaller;

/// Server manager for Lore repository backend
pub struct ServerManager {
    nap_home: std::path::PathBuf,
    http_port: u16,
    health_check_timeout: Duration,
    startup_timeout: Duration,
    retry_interval: Duration,
    max_retries: usize,
}

impl ServerManager {
    /// Create a new server manager
    pub fn new(nap_home: &Path) -> Self {
        Self {
            nap_home: nap_home.to_path_buf(),
            http_port: 41339, // Default Lore HTTP port
            health_check_timeout: Duration::from_secs(5),
            startup_timeout: Duration::from_secs(30),
            retry_interval: Duration::from_secs(1),
            max_retries: 3,
        }
    }

    /// Set custom HTTP port
    pub fn with_http_port(mut self, port: u16) -> Self {
        self.http_port = port;
        self
    }

    /// Ensure Lore is installed
    pub fn ensure_installed(&self) -> Result<()> {
        info!("Checking Lore installation");

        let status =
            verify_lore_installation().context("Failed to verify Lore installation status")?;

        if !status.is_fully_compatible() {
            let message = status.status_message();
            
            // If completely missing, try to install automatically
            if !status.cli_installed || !status.server_installed {
                info!("Lore not detected, attempting automatic installation");
                let installer = LoreInstaller::new(None);
                installer.install_all().context(format!("[{}] Failed to automatically install Lore", error_ids::ERR_LORE_INSTALL_FAILED))?;
                
                // Re-verify after install
                let new_status = verify_lore_installation()?;
                if !new_status.is_fully_compatible() {
                     anyhow::bail!(
                        "[{}] Lore installation failed or is incompatible: {}. \
                         Fix: run 'nap install lore' to manually install.",
                        error_ids::ERR_LORE_INCOMPATIBLE,
                        new_status.status_message()
                    );
                }
                return Ok(());
            }

            // Incompatible but present
            error!(
                installed = status.cli_installed,
                cli_version = status.cli_version.as_ref().map(|v| v.raw.as_str()).unwrap_or("not detected"),
                server_installed = status.server_installed,
                server_version = status.server_version.as_ref().map(|v| v.raw.as_str()).unwrap_or("not detected"),
                pinned = %status.pinned_version,
                "Lore installation incompatible: {}",
                message
            );
            anyhow::bail!(
                "[{}] Lore installation is incompatible: {}. \
                 Required version: {}. \
                 Fix: run 'nap install lore' to install the correct version.",
                error_ids::ERR_LORE_INCOMPATIBLE,
                message,
                status.pinned_version
            );
        }

        info!(
            pinned_version = %status.pinned_version,
            "Lore installation is compatible"
        );
        Ok(())
    }

    /// Ensure Lore is configured
    pub fn ensure_configured(&self) -> Result<()> {
        info!(nap_home = %self.nap_home.display(), "Ensuring Lore configuration");

        // Generate configuration
        let config_files = generate_local_config(&self.nap_home).context(format!(
            "[{}] Failed to generate Lore configuration at '{}'. \
                 Check directory permissions and disk space.",
            error_ids::ERR_LORE_CONFIG_FAILED,
            self.nap_home.display()
        ))?;
        tracing::debug!(config_path = %config_files.config_path.display(), "Lore config generated");

        // Generate certificates
        let cert_dir = self.nap_home.join("lore").join("certs");
        let cert_files = generate_certificates(&cert_dir).context(format!(
            "[{}] Failed to generate Lore certificates at '{}'. \
                 Check directory permissions.",
            error_ids::ERR_LORE_CERT_FAILED,
            cert_dir.display()
        ))?;
        tracing::debug!(
            cert = %cert_files.cert_path.display(),
            key = %cert_files.key_path.display(),
            "Lore certificates generated"
        );

        info!("Lore configuration and certificates are ready");
        Ok(())
    }

    /// Ensure Lore server is running
    pub async fn ensure_running(&self) -> Result<()> {
        info!("Ensuring Lore server is running");

        // Try to acquire lock
        let mut lock = ProcessLock::new(&self.nap_home);
        if !lock.try_acquire()? {
            // Server lock is held - check if daemon is actually healthy
            let daemon_pid = lock
                .read_daemon_pid()
                .unwrap_or(None)
                .map(|p| p.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            info!(
                daemon_pid = %daemon_pid,
                "Server lock already held, verifying daemon health"
            );

            if LoreProcessManager::health_check(self.http_port, self.health_check_timeout).await? {
                info!(daemon_pid = %daemon_pid, "Existing daemon is healthy and running");
                return Ok(());
            }

            warn!(
                daemon_pid = %daemon_pid,
                "Server lock held but daemon health check failed — \
                 the daemon may have crashed. Remove the lock file at '{}' and retry.",
                self.nap_home.join("lore").join("pid").display()
            );
            anyhow::bail!(
                "Server lock is held by daemon PID {} but the server is not responding to health checks. \
                 The daemon may have crashed. Try: nap doctor, or manually remove '{}'.",
                daemon_pid,
                self.nap_home.join("lore").join("pid").display()
            );
        }

        // Start server
        self.start_internal(&mut lock).await?;

        info!("Lore server is running and healthy");
        Ok(())
    }

    /// Start Lore server
    pub async fn start(&self) -> Result<()> {
        info!("Starting Lore server");

        let mut lock = ProcessLock::new(&self.nap_home);
        if !lock.try_acquire()? {
            anyhow::bail!("Server is already running (lock held)");
        }

        self.start_internal(&mut lock).await?;

        info!("Lore server started successfully");
        Ok(())
    }

    /// Internal start implementation
    async fn start_internal(&self, lock: &mut ProcessLock) -> Result<()> {
        let process_manager = LoreProcessManager::new(&self.nap_home);

        // Start the process
        let child = process_manager
            .start()
            .context("Failed to start Lore server process")?;

        // Write the actual daemon PID to the lock file
        let daemon_pid = child.id();
        lock.write_daemon_pid(daemon_pid)
            .context("Failed to write daemon PID to lock file")?;

        tracing::info!(daemon_pid, "Lore daemon spawned, waiting for health check");

        // Wait for server to become healthy
        let mut retries = 0;
        while retries < self.max_retries {
            match crate::server::process::LoreProcessManager::wait_for_healthy(
                self.http_port,
                self.startup_timeout,
                self.retry_interval,
            )
            .await
            {
                Ok(_) => return Ok(()),
                Err(e) => {
                    retries += 1;
                    warn!(
                        attempt = retries,
                        max_retries = self.max_retries,
                        error = %e,
                        "Health check attempt {} of {} failed",
                        retries, self.max_retries
                    );
                    if retries < self.max_retries {
                        tokio::time::sleep(self.retry_interval).await;
                    }
                }
            }
        }

        // Release lock on failure
        lock.release()?;
        anyhow::bail!(
            "[{}] Lore server failed to become healthy after {} retries. \
             Check logs at '{}' for startup errors.",
            error_ids::ERR_LORE_STARTUP_FAILED,
            self.max_retries,
            self.nap_home
                .join("lore")
                .join("logs")
                .join("loreserver.log")
                .display()
        );
    }

    /// Stop Lore server
    pub fn stop(&self) -> Result<()> {
        let lock_file = self.nap_home.join("lore").join("pid");
        if !lock_file.exists() {
            info!(
                "No lock file found at '{}', server may not be running",
                lock_file.display()
            );
            return Ok(());
        }

        let pid_str = std::fs::read_to_string(&lock_file).context(format!(
            "Failed to read PID lock file at '{}'",
            lock_file.display()
        ))?;
        let pid: u32 = pid_str.trim().parse().context(format!(
            "Failed to parse PID '{}' from lock file at '{}'",
            pid_str.trim(),
            lock_file.display()
        ))?;

        info!(pid, "Stopping Lore server process");

        LoreProcessManager::stop(pid).context(format!(
            "Failed to stop Lore server process (PID {}). \
                 The process may have already exited.",
            pid
        ))?;

        // Remove lock file
        std::fs::remove_file(&lock_file).context(format!(
            "Failed to remove lock file at '{}'",
            lock_file.display()
        ))?;

        info!(pid, "Lore server stopped successfully");
        Ok(())
    }

    /// Restart Lore server
    pub async fn restart(&self) -> Result<()> {
        info!("Restarting Lore server");

        self.stop()?;
        tokio::time::sleep(Duration::from_secs(2)).await; // Give it time to stop
        self.start().await?;

        info!("Lore server restarted");
        Ok(())
    }

    /// Get server status
    pub async fn status(&self) -> Result<ServerStatus> {
        let lock_file = self.nap_home.join("lore").join("pid");

        let running = if lock_file.exists() {
            let pid_str =
                std::fs::read_to_string(&lock_file).context("Failed to read lock file")?;
            let pid: u32 = pid_str
                .trim()
                .parse()
                .context("Failed to parse PID from lock file")?;

            LoreProcessManager::is_running(pid)
        } else {
            false
        };

        let healthy = if running {
            LoreProcessManager::health_check(self.http_port, self.health_check_timeout).await?
        } else {
            false
        };

        let configured = self
            .nap_home
            .join("lore")
            .join("config")
            .join("local.toml")
            .exists();

        Ok(ServerStatus {
            running,
            healthy,
            configured,
            http_port: self.http_port,
        })
    }

    /// Perform health check
    pub async fn health_check(&self) -> Result<bool> {
        LoreProcessManager::health_check(self.http_port, self.health_check_timeout).await
    }

    /// Upgrade Lore (not implemented yet)
    pub fn upgrade(&self) -> Result<()> {
        anyhow::bail!("Lore upgrade is not yet implemented");
    }
}

/// Server status information
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub running: bool,
    pub healthy: bool,
    pub configured: bool,
    pub http_port: u16,
}

impl ServerStatus {
    /// Check if server is ready for use
    pub fn is_ready(&self) -> bool {
        self.running && self.healthy && self.configured
    }

    /// Get a human-readable status message
    pub fn status_message(&self) -> String {
        if self.is_ready() {
            "Server is running and healthy".to_string()
        } else if !self.configured {
            "Server is not configured".to_string()
        } else if !self.running {
            "Server is not running".to_string()
        } else {
            "Server is running but not healthy".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_server_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ServerManager::new(temp_dir.path());
        assert_eq!(manager.http_port, 41339);
    }

    #[test]
    fn test_server_manager_custom_port() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ServerManager::new(temp_dir.path()).with_http_port(8080);
        assert_eq!(manager.http_port, 8080);
    }

    #[test]
    fn test_server_status_message() {
        let status = ServerStatus {
            running: true,
            healthy: true,
            configured: true,
            http_port: 41339,
        };
        assert_eq!(status.status_message(), "Server is running and healthy");

        let status = ServerStatus {
            running: false,
            healthy: false,
            configured: false,
            http_port: 41339,
        };
        assert_eq!(status.status_message(), "Server is not configured");
    }
}
