// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! NAP doctor diagnostics and repair
//!
//! Performs comprehensive diagnostics on the NAP SDK environment and
//! provides repair capabilities for common issues.

use anyhow::Result;
use std::path::Path;
use tracing::{error, info};

use super::{
    cert::generate_certificates, config::generate_local_config, install::LoreInstaller,
    manager::ServerManager, version::verify_lore_installation,
};

/// NAP doctor for diagnostics and repair
pub struct NapDoctor {
    nap_home: std::path::PathBuf,
}

impl NapDoctor {
    /// Create a new NAP doctor
    pub fn new(nap_home: &Path) -> Self {
        Self {
            nap_home: nap_home.to_path_buf(),
        }
    }

    /// Run comprehensive diagnostics
    pub async fn diagnose(&self) -> Result<DoctorReport> {
        info!("Running NAP doctor diagnostics");

        let mut checks = vec![];

        // Check NAP configuration
        checks.push(self.check_nap_configuration());

        // Check Lore installation
        checks.push(self.check_lore_installation());

        // Check Lore configuration
        checks.push(self.check_lore_configuration());

        // Check Lore certificates
        checks.push(self.check_lore_certificates());

        // Check Lore server status
        checks.push(self.check_lore_server_status().await);

        // Check store directories
        checks.push(self.check_store_directories());

        // Check provider connectivity
        checks.push(self.check_provider_connectivity().await);

        let report = DoctorReport {
            checks,
            nap_home: self.nap_home.clone(),
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

    /// Check NAP configuration
    fn check_nap_configuration(&self) -> CheckResult {
        let name = "NAP Configuration";

        let config_exists = self.nap_home.exists();
        let config_dir = self.nap_home.join("lore").join("config");
        let lore_config_exists = config_dir.exists();

        if config_exists && lore_config_exists {
            CheckResult {
                name: name.to_string(),
                passed: true,
                message: "NAP configuration exists".to_string(),
                severity: CheckSeverity::Info,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                passed: false,
                message: "NAP configuration missing".to_string(),
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

        let config_path = self.nap_home.join("lore").join("config").join("local.toml");

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

        let cert_dir = self.nap_home.join("lore").join("certs");
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

        let server_manager = ServerManager::new(&self.nap_home);

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

        let immutable_dir = self.nap_home.join("lore").join("store").join("immutable");
        let mutable_dir = self.nap_home.join("lore").join("store").join("mutable");

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

    /// Check provider connectivity
    async fn check_provider_connectivity(&self) -> CheckResult {
        let name = "Provider Connectivity";

        // For now, just check if we can reach localhost
        match reqwest::get("http://127.0.0.1:41339/health_check").await {
            Ok(resp) => {
                if resp.status().is_success() {
                    CheckResult {
                        name: name.to_string(),
                        passed: true,
                        message: "Provider connectivity OK".to_string(),
                        severity: CheckSeverity::Info,
                    }
                } else {
                    CheckResult {
                        name: name.to_string(),
                        passed: false,
                        message: format!("Provider returned status: {}", resp.status()),
                        severity: CheckSeverity::Warning,
                    }
                }
            }
            Err(e) => CheckResult {
                name: name.to_string(),
                passed: false,
                message: format!("Provider connectivity failed: {}", e),
                severity: CheckSeverity::Warning,
            },
        }
    }

    /// Repair a specific check
    async fn repair_check(&self, check: &CheckResult) -> Result<RepairResult> {
        match check.name.as_str() {
            "NAP Configuration" => {
                std::fs::create_dir_all(&self.nap_home)?;
                Ok(RepairResult {
                    check_name: check.name.clone(),
                    success: true,
                    message: "Created NAP home directory".to_string(),
                })
            }
            "Lore Installation" => {
                let install_dir = dirs::home_dir().unwrap().join(".local").join("bin");
                let installer = LoreInstaller::new(&install_dir);
                installer.install_all()?;
                Ok(RepairResult {
                    check_name: check.name.clone(),
                    success: true,
                    message: "Installed Lore CLI and server".to_string(),
                })
            }
            "Lore Configuration" => {
                generate_local_config(&self.nap_home)?;
                Ok(RepairResult {
                    check_name: check.name.clone(),
                    success: true,
                    message: "Generated Lore configuration".to_string(),
                })
            }
            "Lore Certificates" => {
                let cert_dir = self.nap_home.join("lore").join("certs");
                generate_certificates(&cert_dir)?;
                Ok(RepairResult {
                    check_name: check.name.clone(),
                    success: true,
                    message: "Generated Lore certificates".to_string(),
                })
            }
            "Lore Server Status" => {
                let server_manager = ServerManager::new(&self.nap_home);
                server_manager.ensure_running().await?;
                Ok(RepairResult {
                    check_name: check.name.clone(),
                    success: true,
                    message: "Started Lore server".to_string(),
                })
            }
            "Store Directories" => {
                let immutable_dir = self.nap_home.join("lore").join("store").join("immutable");
                let mutable_dir = self.nap_home.join("lore").join("store").join("mutable");
                std::fs::create_dir_all(&immutable_dir)?;
                std::fs::create_dir_all(&mutable_dir)?;
                Ok(RepairResult {
                    check_name: check.name.clone(),
                    success: true,
                    message: "Created store directories".to_string(),
                })
            }
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
    pub nap_home: std::path::PathBuf,
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
        let doctor = NapDoctor::new(temp_dir.path());
        assert_eq!(doctor.nap_home, temp_dir.path());
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
            nap_home: std::path::PathBuf::from("/tmp"),
        };
        assert_eq!(report.overall_health(), HealthStatus::Healthy);
    }
}
