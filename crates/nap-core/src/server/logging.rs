// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Persistent logging for NAP SDK and Lore server
//!
//! Provides structured logging to persistent files for diagnostics and support.

use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::path::Path;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize persistent logging for NAP SDK
pub fn init_persistent_logging(nap_home: &Path) -> Result<()> {
    let logs_dir = nap_home.join("logs");
    std::fs::create_dir_all(&logs_dir).context("Failed to create logs directory")?;

    // NAP SDK log file
    let nap_log_path = logs_dir.join("nap.log");

    // Lore server log file
    let lore_log_path = logs_dir.join("loreserver.log");

    // Initialize tracing subscriber with file output
    let nap_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&nap_log_path)
        .context("Failed to open NAP log file")?;

    let _lore_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&lore_log_path)
        .context("Failed to open Lore server log file")?;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(nap_file))
        .init();

    tracing::info!("Persistent logging initialized");
    tracing::info!("NAP log: {}", nap_log_path.display());
    tracing::info!("Lore server log: {}", lore_log_path.display());

    Ok(())
}

/// Initialize rolling log files with rotation
pub fn init_rolling_logging(nap_home: &Path) -> Result<()> {
    let logs_dir = nap_home.join("logs");
    std::fs::create_dir_all(&logs_dir).context("Failed to create logs directory")?;

    // Rolling file appender for NAP logs (daily rotation)
    let nap_appender = RollingFileAppender::new(Rotation::DAILY, &logs_dir, "nap.log");

    // Rolling file appender for Lore server logs (daily rotation)
    let lore_appender = RollingFileAppender::new(Rotation::DAILY, &logs_dir, "loreserver.log");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(nap_appender))
        .with(fmt::layer().with_writer(lore_appender))
        .init();

    tracing::info!("Rolling logging initialized");
    tracing::info!("Logs directory: {}", logs_dir.display());

    Ok(())
}

/// Get the path to the NAP log file
pub fn nap_log_path(nap_home: &Path) -> std::path::PathBuf {
    nap_home.join("logs").join("nap.log")
}

/// Get the path to the Lore server log file
pub fn lore_log_path(nap_home: &Path) -> std::path::PathBuf {
    nap_home.join("logs").join("loreserver.log")
}

/// Read recent log entries from a file
pub fn read_recent_logs(log_path: &Path, line_count: usize) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(log_path).context("Failed to read log file")?;

    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    let recent_lines = if lines.len() > line_count {
        lines[lines.len() - line_count..].to_vec()
    } else {
        lines
    };

    Ok(recent_lines)
}

/// Tail a log file (return last N lines)
pub fn tail_log(log_path: &Path, line_count: usize) -> Result<Vec<String>> {
    read_recent_logs(log_path, line_count)
}

/// Clear log files
pub fn clear_logs(nap_home: &Path) -> Result<()> {
    let logs_dir = nap_home.join("logs");

    if !logs_dir.exists() {
        return Ok(());
    }

    let nap_log = nap_log_path(nap_home);
    let lore_log = lore_log_path(nap_home);

    if nap_log.exists() {
        std::fs::write(&nap_log, "").context("Failed to clear NAP log")?;
    }

    if lore_log.exists() {
        std::fs::write(&lore_log, "").context("Failed to clear Lore server log")?;
    }

    tracing::info!("Logs cleared");
    Ok(())
}

/// Get log file size in bytes
pub fn log_file_size(log_path: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(log_path).context("Failed to get log file metadata")?;
    Ok(metadata.len())
}

/// Get total size of all log files
pub fn total_log_size(nap_home: &Path) -> Result<u64> {
    let nap_log = nap_log_path(nap_home);
    let lore_log = lore_log_path(nap_home);

    let mut total = 0u64;

    if nap_log.exists() {
        total += log_file_size(&nap_log)?;
    }

    if lore_log.exists() {
        total += log_file_size(&lore_log)?;
    }

    Ok(total)
}

/// Log file information
#[derive(Debug, Clone)]
pub struct LogFileInfo {
    pub path: std::path::PathBuf,
    pub size_bytes: u64,
    pub exists: bool,
}

/// Get information about all log files
pub fn log_files_info(nap_home: &Path) -> Result<Vec<LogFileInfo>> {
    let nap_log = nap_log_path(nap_home);
    let lore_log = lore_log_path(nap_home);

    let mut files = vec![];

    files.push(LogFileInfo {
        path: nap_log.clone(),
        size_bytes: if nap_log.exists() {
            log_file_size(&nap_log)?
        } else {
            0
        },
        exists: nap_log.exists(),
    });

    files.push(LogFileInfo {
        path: lore_log.clone(),
        size_bytes: if lore_log.exists() {
            log_file_size(&lore_log)?
        } else {
            0
        },
        exists: lore_log.exists(),
    });

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_paths() {
        let temp_dir = TempDir::new().unwrap();
        let nap_log = nap_log_path(temp_dir.path());
        let lore_log = lore_log_path(temp_dir.path());

        assert_eq!(nap_log, temp_dir.path().join("logs").join("nap.log"));
        assert_eq!(
            lore_log,
            temp_dir.path().join("logs").join("loreserver.log")
        );
    }

    #[test]
    fn test_log_file_size() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        std::fs::write(&log_path, "test content")?;

        let size = log_file_size(&log_path)?;
        assert_eq!(size, 12);

        Ok(())
    }

    #[test]
    fn test_read_recent_logs() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        let content = "line1\nline2\nline3\nline4\nline5";
        std::fs::write(&log_path, content)?;

        let recent = read_recent_logs(&log_path, 2)?;
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0], "line4");
        assert_eq!(recent[1], "line5");

        Ok(())
    }

    #[test]
    fn test_tail_log() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        let content = "line1\nline2\nline3";
        std::fs::write(&log_path, content)?;

        let tail = tail_log(&log_path, 2)?;
        assert_eq!(tail.len(), 2);
        assert_eq!(tail[0], "line2");
        assert_eq!(tail[1], "line3");

        Ok(())
    }

    #[test]
    fn test_clear_logs() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let logs_dir = temp_dir.path().join("logs");
        std::fs::create_dir_all(&logs_dir)?;

        let nap_log = nap_log_path(temp_dir.path());
        std::fs::write(&nap_log, "some content")?;

        clear_logs(temp_dir.path())?;

        let content = std::fs::read_to_string(&nap_log)?;
        assert_eq!(content, "");

        Ok(())
    }

    #[test]
    fn test_log_files_info() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let logs_dir = temp_dir.path().join("logs");
        std::fs::create_dir_all(&logs_dir)?;

        let nap_log = nap_log_path(temp_dir.path());
        std::fs::write(&nap_log, "test content")?;

        let info = log_files_info(temp_dir.path())?;
        assert_eq!(info.len(), 2);
        assert!(info[0].exists);
        assert_eq!(info[0].size_bytes, 12);

        Ok(())
    }
}
