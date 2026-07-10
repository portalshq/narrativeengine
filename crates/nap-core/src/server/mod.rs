// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Server management for Lore repository backend
//!
//! This module provides server lifecycle management for the Local provider,
//! including certificate generation, configuration generation, and process management.

pub mod cert;
pub mod config;
pub mod doctor;
pub mod install;
pub mod lock;
pub mod logging;
pub mod manager;
pub mod process;
pub mod version;

pub use cert::{generate_certificates, CertificateFiles};
pub use config::{generate_local_config, ConfigFiles};
pub use doctor::{NapDoctor, DoctorReport, RepairReport, CheckResult, CheckSeverity, HealthStatus};
pub use install::{LoreInstaller, VerificationResult};
pub use lock::ProcessLock;
pub use logging::{
    init_persistent_logging, init_rolling_logging, nap_log_path, lore_log_path,
    read_recent_logs, tail_log, clear_logs, log_file_size, total_log_size, log_files_info, LogFileInfo,
};
pub use manager::{ServerManager, ServerStatus};
pub use process::LoreProcessManager;
pub use version::{check_lore_compatibility, detect_lore_version, detect_loreserver_version, verify_lore_installation, LoreInstallationStatus, PINNED_LORE_VERSION};
