// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Persistent logging for PX SDK and Lore server
//!
//! Provides structured logging to persistent files for diagnostics and support.

use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::path::Path;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize persistent logging for PX SDK
pub fn init_persistent_logging(px_home: &Path) -> Result<()> {
    let logs_dir = px_home.join("logs");
    std::fs::create_dir_all(&logs_dir).context("Failed to create logs directory")?;

    // PX SDK log file
    let px_log_path = logs_dir.join("px.log");

    // Lore server log file
    let lore_log_path = logs_dir.join("loreserver.log");

    // Initialize tracing subscriber with file output
    let px_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&px_log_path)
        .context("Failed to open PX log file")?;

    let _lore_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&lore_log_path)
        .context("Failed to open Lore server log file")?;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(px_file))
        .init();

    tracing::info!("Persistent logging initialized");
    tracing::info!("PX log: {}", px_log_path.display());
    tracing::info!("Lore server log: {}", lore_log_path.display());

    Ok(())
}

/// Initialize rolling log files with rotation
pub fn init_rolling_logging(px_home: &Path) -> Result<()> {
    let logs_dir = px_home.join("logs");
    std::fs::create_dir_all(&logs_dir).context("Failed to create logs directory")?;

    // Rolling file appender for PX logs (daily rotation)
    let px_appender = RollingFileAppender::new(Rotation::DAILY, &logs_dir, "px.log");

    // Rolling file appender for Lore server logs (daily rotation)
    let lore_appender = RollingFileAppender::new(Rotation::DAILY, &logs_dir, "loreserver.log");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(px_appender))
        .with(fmt::layer().with_writer(lore_appender))
        .init();

    tracing::info!("Rolling logging initialized");
    tracing::info!("Logs directory: {}", logs_dir.display());

    Ok(())
}

/// Get the path to the PX log file
pub fn px_log_path(px_home: &Path) -> std::path::PathBuf {
    px_home.join("logs").join("px.log")
}

/// Get the path to the Lore server log file
pub fn lore_log_path(px_home: &Path) -> std::path::PathBuf {
    px_home.join("logs").join("loreserver.log")
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
pub fn clear_logs(px_home: &Path) -> Result<()> {
    let logs_dir = px_home.join("logs");

    if !logs_dir.exists() {
        return Ok(());
    }

    let px_log = px_log_path(px_home);
    let lore_log = lore_log_path(px_home);

    if px_log.exists() {
        std::fs::write(&px_log, "").context("Failed to clear PX log")?;
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
pub fn total_log_size(px_home: &Path) -> Result<u64> {
    let px_log = px_log_path(px_home);
    let lore_log = lore_log_path(px_home);

    let mut total = 0u64;

    if px_log.exists() {
        total += log_file_size(&px_log)?;
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
pub fn log_files_info(px_home: &Path) -> Result<Vec<LogFileInfo>> {
    let px_log = px_log_path(px_home);
    let lore_log = lore_log_path(px_home);

    let mut files = vec![];

    files.push(LogFileInfo {
        path: px_log.clone(),
        size_bytes: if px_log.exists() {
            log_file_size(&px_log)?
        } else {
            0
        },
        exists: px_log.exists(),
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
        let px_log = px_log_path(temp_dir.path());
        let lore_log = lore_log_path(temp_dir.path());

        assert_eq!(px_log, temp_dir.path().join("logs").join("px.log"));
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

        let px_log = px_log_path(temp_dir.path());
        std::fs::write(&px_log, "some content")?;

        clear_logs(temp_dir.path())?;

        let content = std::fs::read_to_string(&px_log)?;
        assert_eq!(content, "");

        Ok(())
    }

    #[test]
    fn test_log_files_info() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let logs_dir = temp_dir.path().join("logs");
        std::fs::create_dir_all(&logs_dir)?;

        let px_log = px_log_path(temp_dir.path());
        std::fs::write(&px_log, "test content")?;

        let info = log_files_info(temp_dir.path())?;
        assert_eq!(info.len(), 2);
        assert!(info[0].exists);
        assert_eq!(info[0].size_bytes, 12);

        Ok(())
    }
}
