// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! PX doctor diagnostics and repair
//!
//! Performs comprehensive diagnostics on the PX SDK environment and
//! provides repair capabilities for common issues.

use anyhow::Result;
use std::path::Path;
use tracing::{error, info};

use super::{
    cert::generate_certificates, config::generate_local_config, install::LoreInstaller,
    manager::ServerManager, version::verify_lore_installation,
};
use crate::vcs::VcsBackend;

/// PX doctor for diagnostics and repair
pub struct PxDoctor {
    px_home: std::path::PathBuf,
}

impl PxDoctor {
    /// Create a new PX doctor
    pub fn new(px_home: &Path) -> Self {
        Self {
            px_home: px_home.to_path_buf(),
        }
    }

    /// Run comprehensive diagnostics — provider-aware so `px doctor` is useful
    /// for both local and remote deployments. Local checks (store dirs,
    /// certs, daemon) are skipped for `remote`/`portals-cloud` where they are
    /// irrelevant; remote connectivity and stale-checkout detection run instead.
    pub async fn diagnose(&self) -> Result<DoctorReport> {
        info!("Running PX doctor diagnostics");

        let provider_type = {
            let mut pm = crate::provider::ProviderManager::new(&self.px_home);
            match pm.load_configured_provider() {
                Ok(Some(p)) => Some(p.provider_type()),
                Ok(None) => None,
                Err(_) => None,
            }
        };
        let is_remote = matches!(
            provider_type,
            Some(crate::provider::ProviderType::Remote)
                | Some(crate::provider::ProviderType::PortalsCloud)
        );

        let mut checks = vec![];

        // PX home + provider config is always relevant.
        checks.push(self.check_px_configuration());
        checks.push(self.check_provider_configuration());

        // Lore CLI is required for all providers (push/pull/resolve).
        checks.push(self.check_lore_installation());

        if is_remote {
            // Remote: don't require local daemon / certs / store.
            checks.push(self.check_provider_connectivity().await);
            checks.push(self.check_repository_remote_consistency());
        } else {
            // Local (or unconfigured): full local-server health.
            checks.push(self.check_lore_configuration());
            checks.push(self.check_lore_certificates());
            checks.push(self.check_lore_server_status().await);
            checks.push(self.check_store_directories());
            checks.push(self.check_provider_connectivity().await);
            checks.push(self.check_repository_remote_consistency());
        }

        let report = DoctorReport {
            checks,
            px_home: self.px_home.clone(),
        };

        info!("Diagnostics complete: {} checks", report.checks.len());
        Ok(report)
    }

    /// Repair detected issues
    pub async fn repair(&self, report: &DoctorReport) -> Result<RepairReport> {
        info!("Starting repair based on diagnostics");

        let mut repairs = vec![];

        for check in &report.checks {
            if !check.passed {
                match self.repair_check(check).await {
                    Ok(repair_result) => {
                        repairs.push(repair_result);
                    }
                    Err(e) => {
                        error!("Failed to repair {}: {}", check.name, e);
                        repairs.push(RepairResult {
                            check_name: check.name.clone(),
                            success: false,
                            message: format!("Repair failed: {}", e),
                        });
                    }
                }
            }
        }

        let repair_report = RepairReport { repairs };
        info!(
            "Repair complete: {} repairs attempted",
            repair_report.repairs.len()
        );
        Ok(repair_report)
    }

    /// Check PX configuration — only ensures the PX home directory exists.
    /// Lore-specific layout is checked separately via `check_lore_configuration`
    /// so remote providers don't fail this check when local store isn't needed.
    fn check_px_configuration(&self) -> CheckResult {
        let name = "PX Configuration";

        if self.px_home.exists() {
            CheckResult {
                name: name.to_string(),
                passed: true,
                message: "PX home exists".to_string(),
                severity: CheckSeverity::Info,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                passed: false,
                message: "PX home missing".to_string(),
                severity: CheckSeverity::Error,
            }
        }
    }

    /// Check Lore installation
    fn check_lore_installation(&self) -> CheckResult {
        let name = "Lore Installation";

        match verify_lore_installation() {
            Ok(status) => {
                if status.is_fully_compatible() {
                    CheckResult {
                        name: name.to_string(),
                        passed: true,
                        message: format!("Lore {} installed and compatible", status.pinned_version),
                        severity: CheckSeverity::Info,
                    }
                } else {
                    CheckResult {
                        name: name.to_string(),
                        passed: false,
                        message: status.status_message(),
                        severity: CheckSeverity::Error,
                    }
                }
            }
            Err(e) => CheckResult {
                name: name.to_string(),
                passed: false,
                message: format!("Failed to check Lore installation: {}", e),
                severity: CheckSeverity::Error,
            },
        }
    }

    /// Check Lore configuration
    fn check_lore_configuration(&self) -> CheckResult {
        let name = "Lore Configuration";

        let config_path = self.px_home.join("lore").join("config").join("local.toml");

        if config_path.exists() {
            CheckResult {
                name: name.to_string(),
                passed: true,
                message: "Lore configuration exists".to_string(),
                severity: CheckSeverity::Info,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                passed: false,
                message: "Lore configuration missing".to_string(),
                severity: CheckSeverity::Warning,
            }
        }
    }

    /// Check Lore certificates
    fn check_lore_certificates(&self) -> CheckResult {
        let name = "Lore Certificates";

        let cert_dir = self.px_home.join("lore").join("certs");
        let cert_path = cert_dir.join("cert.pem");
        let key_path = cert_dir.join("key.pem");

        if cert_path.exists() && key_path.exists() {
            CheckResult {
                name: name.to_string(),
                passed: true,
                message: "Lore certificates exist".to_string(),
                severity: CheckSeverity::Info,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                passed: false,
                message: "Lore certificates missing".to_string(),
                severity: CheckSeverity::Warning,
            }
        }
    }

    /// Check Lore server status
    async fn check_lore_server_status(&self) -> CheckResult {
        let name = "Lore Server Status";

        let server_manager = ServerManager::new(&self.px_home);

        match server_manager.status().await {
            Ok(status) => {
                if status.is_ready() {
                    CheckResult {
                        name: name.to_string(),
                        passed: true,
                        message: format!("Lore server running on port {}", status.http_port),
                        severity: CheckSeverity::Info,
                    }
                } else {
                    CheckResult {
                        name: name.to_string(),
                        passed: false,
                        message: status.status_message(),
                        severity: CheckSeverity::Warning,
                    }
                }
            }
            Err(e) => CheckResult {
                name: name.to_string(),
                passed: false,
                message: format!("Failed to check server status: {}", e),
                severity: CheckSeverity::Error,
            },
        }
    }

    /// Check store directories
    fn check_store_directories(&self) -> CheckResult {
        let name = "Store Directories";

        let immutable_dir = self.px_home.join("lore").join("store").join("immutable");
        let mutable_dir = self.px_home.join("lore").join("store").join("mutable");

        if immutable_dir.exists() && mutable_dir.exists() {
            CheckResult {
                name: name.to_string(),
                passed: true,
                message: "Store directories exist".to_string(),
                severity: CheckSeverity::Info,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                passed: false,
                message: "Store directories missing".to_string(),
                severity: CheckSeverity::Warning,
            }
        }
    }

    /// Check provider configuration is present and valid.
    fn check_provider_configuration(&self) -> CheckResult {
        let name = "Provider Configuration";
        let mut pm = crate::provider::ProviderManager::new(&self.px_home);
        match pm.load_configured_provider() {
            Ok(Some(provider)) => CheckResult {
                name: name.to_string(),
                passed: true,
                message: format!(
                    "Provider '{}' ({}) configured",
                    provider.name(),
                    provider.provider_type().as_str()
                ),
                severity: CheckSeverity::Info,
            },
            Ok(None) => CheckResult {
                name: name.to_string(),
                passed: false,
                message: "No provider configured; run 'px configure --provider <local|remote|portals-cloud>' or 'px init --provider ...'".to_string(),
                severity: CheckSeverity::Warning,
            },
            Err(e) => CheckResult {
                name: name.to_string(),
                passed: false,
                message: format!("Provider configuration invalid: {e}"),
                severity: CheckSeverity::Error,
            },
        }
    }

    /// Check provider connectivity — uses the configured provider's own
    /// health_probe (local daemon vs remote HTTP/tonic), not hard-coded localhost.
    async fn check_provider_connectivity(&self) -> CheckResult {
        let name = "Provider Connectivity";
        let mut pm = crate::provider::ProviderManager::new(&self.px_home);
        let provider = match pm.load_configured_provider() {
            Ok(Some(p)) => p,
            Ok(None) => {
                return CheckResult {
                    name: name.to_string(),
                    passed: false,
                    message: "No provider configured".to_string(),
                    severity: CheckSeverity::Warning,
                };
            }
            Err(e) => {
                return CheckResult {
                    name: name.to_string(),
                    passed: false,
                    message: format!("Provider configuration invalid: {e}"),
                    severity: CheckSeverity::Error,
                };
            }
        };
        match provider.health_check().await {
            Ok(true) => CheckResult {
                name: name.to_string(),
                passed: true,
                message: format!("Provider '{}' reachable", provider.name()),
                severity: CheckSeverity::Info,
            },
            Ok(false) => CheckResult {
                name: name.to_string(),
                passed: false,
                message: format!("Provider '{}' not reachable", provider.name()),
                severity: CheckSeverity::Warning,
            },
            Err(e) => CheckResult {
                name: name.to_string(),
                passed: false,
                message: format!("Provider connectivity failed: {e}"),
                severity: CheckSeverity::Warning,
            },
        }
    }

    /// Detect checkouts that still track a different Lore server than the
    /// current provider (e.g. Tailscale 100.x vs LAN 192.168.x after
    /// provider.toml was edited). This is the class of bug that surfaces as
    /// `gRPC connection to http://100.105.14.118:41337/: transport error`.
    fn check_repository_remote_consistency(&self) -> CheckResult {
        let name = "Repository Remote Consistency";
        let mut pm = crate::provider::ProviderManager::new(&self.px_home);
        let provider = match pm.load_configured_provider() {
            Ok(Some(p)) => p,
            Ok(None) => {
                return CheckResult {
                    name: name.to_string(),
                    passed: true,
                    message: "No provider configured; skip checkout consistency check".to_string(),
                    severity: CheckSeverity::Info,
                };
            }
            Err(_) => {
                return CheckResult {
                    name: name.to_string(),
                    passed: false,
                    message: "Provider configuration invalid; fix provider.toml".to_string(),
                    severity: CheckSeverity::Error,
                };
            }
        };
        let configured_url = match provider.lore_url_base() {
            Ok(u) => u,
            Err(e) => {
                return CheckResult {
                    name: name.to_string(),
                    passed: false,
                    message: format!("Failed to resolve provider URL: {e}"),
                    severity: CheckSeverity::Error,
                };
            }
        };
        let mut mismatched = Vec::new();
        let mut checked = 0usize;
        let Ok(entries) = std::fs::read_dir(&self.px_home) else {
            return CheckResult {
                name: name.to_string(),
                passed: true,
                message: "PX home not readable; skip checkout check".to_string(),
                severity: CheckSeverity::Info,
            };
        };
        for entry in entries.flatten() {
            let repo_path = entry.path();
            if !repo_path.is_dir() {
                continue;
            }
            if !repo_path.join("repository.yaml").exists() {
                continue;
            }
            // Only consider real Lore checkouts (have .lore).
            if !repo_path.join(".lore").exists() {
                continue;
            }
            checked += 1;
            // Test hook: if `.mock_remote_url` exists (used by unit tests to
            // avoid requiring the `lore` binary), use it instead of calling the
            // CLI. This has no effect in production.
            let descriptor_remote_url =
                if let Ok(mock) = std::fs::read_to_string(repo_path.join(".mock_remote_url")) {
                    mock.trim().to_string()
                } else {
                    let backend = crate::vcs_lore::LoreBackend::from_px_home(&self.px_home);
                    // Repository descriptor requires lore CLI; if it fails we treat that
                    // as a separate lore-installation issue, not a consistency failure.
                    match backend.repository_descriptor(&repo_path) {
                        Ok(d) => d.remote_url,
                        Err(_) => continue,
                    }
                };
            if descriptor_remote_url.is_empty() {
                continue;
            }
            if !crate::provider::http::same_server(&descriptor_remote_url, &configured_url) {
                let repo_name = repo_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                mismatched.push(format!(
                    "{} tracks {} but provider is {}",
                    repo_name, descriptor_remote_url, configured_url
                ));
            }
        }
        if mismatched.is_empty() {
            CheckResult {
                name: name.to_string(),
                passed: true,
                message: if checked == 0 {
                    "No Lore checkouts to verify".to_string()
                } else {
                    format!("All {checked} checkout(s) match provider {configured_url}")
                },
                severity: CheckSeverity::Info,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                passed: false,
                message: format!(
                    "Stale checkout remote(s): {}. Provider is {configured_url}. Fix: re-clone affected repositories from the new server (e.g. `mv <repo> <repo>.bak && px pull lore://<new-host>:41337/<repo>`) or run with updated provider. See `px status` and `px configure`.",
                    mismatched.join("; ")
                ),
                severity: CheckSeverity::Error,
            }
        }
    }

    /// Repair a specific check
    async fn repair_check(&self, check: &CheckResult) -> Result<RepairResult> {
        match check.name.as_str() {
            "PX Configuration" => {
                std::fs::create_dir_all(&self.px_home)?;
                Ok(RepairResult {
                    check_name: check.name.clone(),
                    success: true,
                    message: "Created PX home directory".to_string(),
                })
            }
            "Provider Configuration" => Ok(RepairResult {
                check_name: check.name.clone(),
                success: false,
                message: "Fix provider.toml manually or run 'px configure --provider <local|remote|portals-cloud> [--remote-url lore://host:41337]'".to_string(),
            }),
            "Lore Installation" => {
                let installer = LoreInstaller::new(None);
                installer.install_all()?;
                Ok(RepairResult {
                    check_name: check.name.clone(),
                    success: true,
                    message: "Installed Lore CLI and server".to_string(),
                })
            }
            "Lore Configuration" => {
                // Only meaningful for local provider.
                let provider_type = {
                    let mut pm = crate::provider::ProviderManager::new(&self.px_home);
                    pm.load_configured_provider()
                        .ok()
                        .flatten()
                        .map(|p| p.provider_type())
                };
                if matches!(
                    provider_type,
                    Some(crate::provider::ProviderType::Remote)
                        | Some(crate::provider::ProviderType::PortalsCloud)
                ) {
                    return Ok(RepairResult {
                        check_name: check.name.clone(),
                        success: false,
                        message: "Local Lore configuration not used for remote provider".to_string(),
                    });
                }
                generate_local_config(&self.px_home)?;
                Ok(RepairResult {
                    check_name: check.name.clone(),
                    success: true,
                    message: "Generated Lore configuration".to_string(),
                })
            }
            "Lore Certificates" => {
                let provider_type = {
                    let mut pm = crate::provider::ProviderManager::new(&self.px_home);
                    pm.load_configured_provider()
                        .ok()
                        .flatten()
                        .map(|p| p.provider_type())
                };
                if matches!(
                    provider_type,
                    Some(crate::provider::ProviderType::Remote)
                        | Some(crate::provider::ProviderType::PortalsCloud)
                ) {
                    return Ok(RepairResult {
                        check_name: check.name.clone(),
                        success: false,
                        message: "Local certificates not used for remote provider".to_string(),
                    });
                }
                let cert_dir = self.px_home.join("lore").join("certs");
                generate_certificates(&cert_dir)?;
                Ok(RepairResult {
                    check_name: check.name.clone(),
                    success: true,
                    message: "Generated Lore certificates".to_string(),
                })
            }
            "Lore Server Status" => {
                let provider_type = {
                    let mut pm = crate::provider::ProviderManager::new(&self.px_home);
                    pm.load_configured_provider()
                        .ok()
                        .flatten()
                        .map(|p| p.provider_type())
                };
                if matches!(
                    provider_type,
                    Some(crate::provider::ProviderType::Remote)
                        | Some(crate::provider::ProviderType::PortalsCloud)
                ) {
                    return Ok(RepairResult {
                        check_name: check.name.clone(),
                        success: false,
                        message: "Local daemon not used for remote provider; verify remote server health".to_string(),
                    });
                }
                let server_manager = ServerManager::new(&self.px_home);
                server_manager.ensure_running().await?;
                Ok(RepairResult {
                    check_name: check.name.clone(),
                    success: true,
                    message: "Started Lore server".to_string(),
                })
            }
            "Store Directories" => {
                let provider_type = {
                    let mut pm = crate::provider::ProviderManager::new(&self.px_home);
                    pm.load_configured_provider()
                        .ok()
                        .flatten()
                        .map(|p| p.provider_type())
                };
                if matches!(
                    provider_type,
                    Some(crate::provider::ProviderType::Remote)
                        | Some(crate::provider::ProviderType::PortalsCloud)
                ) {
                    return Ok(RepairResult {
                        check_name: check.name.clone(),
                        success: false,
                        message: "Local store not used for remote provider".to_string(),
                    });
                }
                let immutable_dir = self.px_home.join("lore").join("store").join("immutable");
                let mutable_dir = self.px_home.join("lore").join("store").join("mutable");
                std::fs::create_dir_all(&immutable_dir)?;
                std::fs::create_dir_all(&mutable_dir)?;
                Ok(RepairResult {
                    check_name: check.name.clone(),
                    success: true,
                    message: "Created store directories".to_string(),
                })
            }
            "Provider Connectivity" => Ok(RepairResult {
                check_name: check.name.clone(),
                success: false,
                message: "Verify network, provider URL (px status / px configure), and remote server health".to_string(),
            }),
            "Repository Remote Consistency" => Ok(RepairResult {
                check_name: check.name.clone(),
                success: false,
                message: "Re-clone stale repositories from the new server (mv <repo> <repo>.bak && px pull lore://<new-host>:41337/<repo>) after confirming 'px status'".to_string(),
            }),
            _ => Ok(RepairResult {
                check_name: check.name.clone(),
                success: false,
                message: "No repair available for this check".to_string(),
            }),
        }
    }
}

/// Result of a diagnostic check
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub severity: CheckSeverity,
}

/// Severity level of a check
#[derive(Debug, Clone, PartialEq)]
pub enum CheckSeverity {
    Info,
    Warning,
    Error,
}

/// Complete diagnostic report
#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub checks: Vec<CheckResult>,
    pub px_home: std::path::PathBuf,
}

impl DoctorReport {
    /// Get overall health status
    pub fn overall_health(&self) -> HealthStatus {
        let has_errors = self
            .checks
            .iter()
            .any(|c| c.severity == CheckSeverity::Error && !c.passed);
        let has_warnings = self
            .checks
            .iter()
            .any(|c| c.severity == CheckSeverity::Warning && !c.passed);

        if has_errors {
            HealthStatus::Unhealthy
        } else if has_warnings {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get a summary message
    pub fn summary(&self) -> String {
        let passed = self.checks.iter().filter(|c| c.passed).count();
        let total = self.checks.len();
        format!("{} / {} checks passed", passed, total)
    }
}

/// Overall health status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Result of a repair operation
#[derive(Debug, Clone)]
pub struct RepairResult {
    pub check_name: String,
    pub success: bool,
    pub message: String,
}

/// Complete repair report
#[derive(Debug, Clone)]
pub struct RepairReport {
    pub repairs: Vec<RepairResult>,
}

impl RepairReport {
    /// Get number of successful repairs
    pub fn successful_count(&self) -> usize {
        self.repairs.iter().filter(|r| r.success).count()
    }

    /// Get number of failed repairs
    pub fn failed_count(&self) -> usize {
        self.repairs.iter().filter(|r| !r.success).count()
    }

    /// Get summary message
    pub fn summary(&self) -> String {
        format!(
            "{} successful, {} failed",
            self.successful_count(),
            self.failed_count()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_doctor_creation() {
        let temp_dir = TempDir::new().unwrap();
        let doctor = PxDoctor::new(temp_dir.path());
        assert_eq!(doctor.px_home, temp_dir.path());
    }

    #[test]
    fn test_check_result() {
        let check = CheckResult {
            name: "Test Check".to_string(),
            passed: true,
            message: "Test passed".to_string(),
            severity: CheckSeverity::Info,
        };
        assert!(check.passed);
        assert_eq!(check.severity, CheckSeverity::Info);
    }

    #[test]
    fn test_health_status() {
        let report = DoctorReport {
            checks: vec![
                CheckResult {
                    name: "Check 1".to_string(),
                    passed: true,
                    message: "OK".to_string(),
                    severity: CheckSeverity::Info,
                },
                CheckResult {
                    name: "Check 2".to_string(),
                    passed: true,
                    message: "OK".to_string(),
                    severity: CheckSeverity::Info,
                },
            ],
            px_home: std::path::PathBuf::from("/tmp"),
        };
        assert_eq!(report.overall_health(), HealthStatus::Healthy);
    }

    #[test]
    fn test_repository_remote_consistency_passes_when_match() {
        let tmp = TempDir::new().unwrap();
        let px_home = tmp.path();
        std::fs::write(
            px_home.join("provider.toml"),
            "provider_type = \"remote\"\nremote_url = \"lore://192.168.0.27:41337\"\nworkspace_id = \"default\"\n",
        )
        .unwrap();
        let repo = px_home.join("my-repo");
        std::fs::create_dir_all(repo.join(".lore")).unwrap();
        std::fs::write(
            repo.join("repository.yaml"),
            "id: px://my-repo/world/my-repo\n",
        )
        .unwrap();
        std::fs::write(
            repo.join(".mock_remote_url"),
            "lore://192.168.0.27:41337/my-repo",
        )
        .unwrap();

        let doctor = PxDoctor::new(px_home);
        let check = doctor.check_repository_remote_consistency();
        assert!(
            check.passed,
            "matching remote should pass: {}",
            check.message
        );
        assert_eq!(check.severity, CheckSeverity::Info);
    }

    #[test]
    fn test_repository_remote_consistency_fails_when_stale() {
        let tmp = TempDir::new().unwrap();
        let px_home = tmp.path();
        std::fs::write(
            px_home.join("provider.toml"),
            "provider_type = \"remote\"\nremote_url = \"lore://192.168.0.27:41337\"\nworkspace_id = \"default\"\n",
        )
        .unwrap();
        let repo = px_home.join("25th-chapter");
        std::fs::create_dir_all(repo.join(".lore")).unwrap();
        std::fs::write(
            repo.join("repository.yaml"),
            "id: px://25th-chapter/world/25th-chapter\n",
        )
        .unwrap();
        std::fs::write(
            repo.join(".mock_remote_url"),
            "lore://100.105.14.118:41337/25th-chapter",
        )
        .unwrap();

        let doctor = PxDoctor::new(px_home);
        let check = doctor.check_repository_remote_consistency();
        assert!(!check.passed, "stale remote should fail");
        assert_eq!(check.severity, CheckSeverity::Error);
        assert!(
            check.message.contains("Stale checkout"),
            "message: {}",
            check.message
        );
        assert!(check.message.contains("100.105.14.118"));
        assert!(check.message.contains("192.168.0.27"));
    }

    #[test]
    fn test_repository_remote_consistency_ignores_non_lore_dirs() {
        let tmp = TempDir::new().unwrap();
        let px_home = tmp.path();
        std::fs::write(
            px_home.join("provider.toml"),
            "provider_type = \"remote\"\nremote_url = \"lore://192.168.0.27:41337\"\nworkspace_id = \"default\"\n",
        )
        .unwrap();
        let repo = px_home.join("not-a-repo");
        std::fs::create_dir_all(&repo).unwrap();
        // No repository.yaml / .lore — should be ignored.
        std::fs::write(repo.join("some.txt"), "hello").unwrap();

        let doctor = PxDoctor::new(px_home);
        let check = doctor.check_repository_remote_consistency();
        assert!(check.passed);
        assert!(check.message.contains("No Lore checkouts") || check.message.contains("All 0"));
    }

    #[tokio::test]
    async fn test_doctor_diagnose_is_provider_aware_for_remote() {
        let tmp = TempDir::new().unwrap();
        let px_home = tmp.path();
        std::fs::write(
            px_home.join("provider.toml"),
            "provider_type = \"remote\"\nremote_url = \"lore://192.168.0.27:41337\"\nworkspace_id = \"default\"\n",
        )
        .unwrap();
        let repo = px_home.join("my-repo");
        std::fs::create_dir_all(repo.join(".lore")).unwrap();
        std::fs::write(
            repo.join("repository.yaml"),
            "id: px://my-repo/world/my-repo\n",
        )
        .unwrap();
        std::fs::write(
            repo.join(".mock_remote_url"),
            "lore://192.168.0.27:41337/my-repo",
        )
        .unwrap();

        let doctor = PxDoctor::new(px_home);
        let report = doctor.diagnose().await.unwrap();
        // Remote should not include local-only checks like Store Directories / Lore Certificates.
        let names: Vec<_> = report.checks.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"Provider Configuration"));
        assert!(names.contains(&"Repository Remote Consistency"));
        assert!(names.contains(&"Provider Connectivity"));
        // Local-only checks must be absent for remote.
        assert!(
            !names.contains(&"Store Directories"),
            "remote diagnose should skip local store checks"
        );
        assert!(!names.contains(&"Lore Certificates"));
    }
}
