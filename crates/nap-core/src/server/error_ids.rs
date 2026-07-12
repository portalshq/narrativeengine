// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT

//! Error IDs for diagnostic tracking.
//!
//! These IDs should be used in all error contexts to allow Sentry to aggregate
//! and track specific failure modes in production.

/// Lore download failed
pub const ERR_LORE_DOWNLOAD_FAILED: &str = "NAP-ERR-001";
/// Lore install script failed
pub const ERR_LORE_INSTALL_FAILED: &str = "NAP-ERR-002";
/// Lore compatibility check failed
pub const ERR_LORE_INCOMPATIBLE: &str = "NAP-ERR-003";
/// Lore configuration generation failed
pub const ERR_LORE_CONFIG_FAILED: &str = "NAP-ERR-004";
/// Lore certificate generation failed
pub const ERR_LORE_CERT_FAILED: &str = "NAP-ERR-005";
/// Lore process failed to start/become healthy
pub const ERR_LORE_STARTUP_FAILED: &str = "NAP-ERR-006";
