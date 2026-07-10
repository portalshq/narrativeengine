// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Lore server process management
//!
//! Cross-platform detached process launch and lifecycle management for Lore server.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Lore server process manager
pub struct LoreProcessManager {
    #[allow(dead_code)]
    nap_home: std::path::PathBuf,
    log_path: std::path::PathBuf,
    config_path: std::path::PathBuf,
}

impl LoreProcessManager {
    /// Create a new Lore process manager
    pub fn new(nap_home: &Path) -> Self {
        let config_path = nap_home.join("lore").join("config").join("local.toml");
        let log_path = nap_home.join("lore").join("logs").join("loreserver.log");

        Self {
            nap_home: nap_home.to_path_buf(),
            config_path,
            log_path,
        }
    }

    /// Start Lore server as a detached background process
    pub fn start(&self) -> Result<Child> {
        // Ensure log directory exists
        if let Some(parent) = self.log_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create log directory")?;
        }

        // Verify configuration exists
        if !self.config_path.exists() {
            anyhow::bail!(
                "Lore configuration not found at '{}'. \
                 Run 'nap init' or ensure configuration generation has been completed \
                 before starting the Lore server.",
                self.config_path.display()
            );
        }

        // Open log file for output
        let log_file =
            std::fs::File::create(&self.log_path).context("Failed to create log file")?;

        tracing::info!(
            config = %self.config_path.display(),
            log = %self.log_path.display(),
            "Starting Lore server"
        );

        // Launch Lore server with configuration
        let child = Command::new("loreserver")
            .arg("--config")
            .arg(&self.config_path)
            .stdout(Stdio::from(log_file.try_clone()?))
            .stderr(Stdio::from(log_file))
            .spawn()
            .context("Failed to start Lore server. Is loreserver installed and on PATH?")?;

        tracing::info!(pid = child.id(), "Lore server started successfully");

        Ok(child)
    }

    /// Stop Lore server by PID
    pub fn stop(pid: u32) -> Result<()> {
        tracing::info!(pid, "Stopping Lore server");

        #[cfg(unix)]
        {
            use nix::sys::signal::kill;
            use nix::unistd::Pid;

            kill(Pid::from_raw(pid as i32), nix::sys::signal::Signal::SIGTERM)
                .context("Failed to send SIGTERM to Lore server")?;
        }

        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::CloseHandle;
            use windows_sys::Win32::System::Threading::{
                OpenProcess, PROCESS_TERMINATE, TerminateProcess,
            };

            unsafe {
                let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
                if handle == 0 {
                    anyhow::bail!("Failed to open process handle for PID {}", pid);
                }
                let result = TerminateProcess(handle, 1);
                CloseHandle(handle);
                if result == 0 {
                    anyhow::bail!("Failed to terminate process with PID {}", pid);
                }
            }
        }

        tracing::info!(pid, "Lore server stopped");
        Ok(())
    }

    /// Check if Lore server is running by PID
    pub fn is_running(pid: u32) -> bool {
        #[cfg(unix)]
        {
            use nix::sys::signal::kill;
            use nix::unistd::Pid;

            kill(Pid::from_raw(pid as i32), None).is_ok()
        }

        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::CloseHandle;
            use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};

            unsafe {
                let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
                if handle == 0 {
                    return false;
                }
                CloseHandle(handle);
                true
            }
        }
    }

    /// Perform health check on Lore server
    pub async fn health_check(port: u16, timeout: Duration) -> Result<bool> {
        let url = format!("http://127.0.0.1:{}/health_check", port);

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .context("Failed to build HTTP client for Lore health check")?;

        let response = client.get(&url).send().await;

        match response {
            Ok(resp) => {
                let is_healthy = resp.status().is_success();
                if is_healthy {
                    tracing::debug!(
                        port,
                        url = %url,
                        "Lore server health check passed"
                    );
                } else {
                    tracing::warn!(
                        port,
                        url = %url,
                        status = %resp.status(),
                        "Lore server health check returned non-success status"
                    );
                }
                Ok(is_healthy)
            }
            Err(e) => {
                if e.is_timeout() {
                    tracing::debug!(
                        port,
                        url = %url,
                        timeout_ms = timeout.as_millis(),
                        "Lore server health check timed out after {:?} — server may still be starting",
                        timeout
                    );
                } else if e.is_connect() {
                    tracing::debug!(
                        port,
                        url = %url,
                        "Lore server health check connection refused — server is not listening on port {}",
                        port
                    );
                } else {
                    tracing::debug!(
                        port,
                        url = %url,
                        error = %e,
                        "Lore server health check failed"
                    );
                }
                Ok(false)
            }
        }
    }

    /// Wait for Lore server to become healthy
    pub async fn wait_for_healthy(
        port: u16,
        timeout: Duration,
        retry_interval: Duration,
    ) -> Result<()> {
        let start = std::time::Instant::now();
        let mut attempt = 0;

        while start.elapsed() < timeout {
            attempt += 1;
            tracing::debug!(
                attempt,
                elapsed_ms = start.elapsed().as_millis(),
                timeout_ms = timeout.as_millis(),
                "Health check attempt for Lore server on port {}",
                port
            );

            if Self::health_check(port, retry_interval).await? {
                tracing::info!(
                    attempt,
                    elapsed_ms = start.elapsed().as_millis(),
                    port,
                    "Lore server became healthy after {} attempts ({:?})",
                    attempt,
                    start.elapsed()
                );
                return Ok(());
            }
            tokio::time::sleep(retry_interval).await;
        }

        anyhow::bail!(
            "Lore server on port {} did not become healthy within {:?} ({} attempts). \
             Possible causes: server crash during startup, port conflict, \
             configuration error, or insufficient resources. \
             Check logs for details.",
            port,
            timeout,
            attempt
        );
    }

    /// Get the log file path
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }

    /// Get the configuration path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_process_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LoreProcessManager::new(temp_dir.path());

        assert_eq!(
            manager.config_path(),
            temp_dir
                .path()
                .join("lore")
                .join("config")
                .join("local.toml")
        );
        assert_eq!(
            manager.log_path(),
            temp_dir
                .path()
                .join("lore")
                .join("logs")
                .join("loreserver.log")
        );
    }

    #[test]
    fn test_is_running() {
        // Test with current process PID
        let current_pid = std::process::id();
        assert!(LoreProcessManager::is_running(current_pid));

        // Test with invalid PID
        assert!(!LoreProcessManager::is_running(999999));
    }
}
