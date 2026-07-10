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

pub use cert::{CertificateFiles, generate_certificates};
pub use config::{ConfigFiles, generate_local_config};
pub use doctor::{CheckResult, CheckSeverity, DoctorReport, HealthStatus, NapDoctor, RepairReport};
pub use install::{LoreInstaller, VerificationResult};
pub use lock::ProcessLock;
pub use logging::{
    LogFileInfo, clear_logs, init_persistent_logging, init_rolling_logging, log_file_size,
    log_files_info, lore_log_path, nap_log_path, read_recent_logs, tail_log, total_log_size,
};
pub use manager::{ServerManager, ServerStatus};
pub use process::LoreProcessManager;
pub use version::{
    LoreInstallationStatus, PINNED_LORE_VERSION, check_lore_compatibility, detect_lore_version,
    detect_loreserver_version, verify_lore_installation,
};
