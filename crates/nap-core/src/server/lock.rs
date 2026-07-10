// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Cross-platform process locking mechanism
//!
//! Ensures only one local Lore daemon is started per NAP home directory
//! using PID file-based locking with automatic cleanup.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process;

/// Process lock to prevent multiple concurrent daemon startups
pub struct ProcessLock {
    lock_file: std::path::PathBuf,
    acquired: bool,
}

impl ProcessLock {
    /// Create a new process lock for the given NAP home directory
    pub fn new(nap_home: &Path) -> Self {
        let lock_file = nap_home.join("lore").join("pid");
        Self {
            lock_file,
            acquired: false,
        }
    }

    /// Try to acquire the lock
    ///
    /// Returns Ok(true) if lock was acquired (no other daemon running),
    /// Ok(false) if already held by another running process.
    ///
    /// **Important**: This only checks for an existing running daemon.
    /// Call [`write_daemon_pid`] after spawning the daemon to record its PID.
    pub fn try_acquire(&mut self) -> Result<bool> {
        // Create lock directory if needed
        if let Some(parent) = self.lock_file.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create lock directory for process lock")?;
        }

        // Check if lock file exists
        if self.lock_file.exists() {
            // Read existing PID
            let existing_pid = fs::read_to_string(&self.lock_file)
                .with_context(|| format!("Failed to read PID lock file at {}", self.lock_file.display()))?;
            
            let existing_pid: u32 = existing_pid
                .trim()
                .parse()
                .with_context(|| format!("Failed to parse PID from lock file at {}", self.lock_file.display()))?;

            // Check if process is still running
            if self.is_process_running(existing_pid) {
                tracing::warn!(
                    pid = existing_pid,
                    lock_file = %self.lock_file.display(),
                    "Lore server is already running (lock held by daemon PID {})",
                    existing_pid
                );
                return Ok(false);
            } else {
                // Stale lock - daemon process died without cleanup
                tracing::info!(
                    pid = existing_pid,
                    lock_file = %self.lock_file.display(),
                    "Removing stale lock file (PID {} no longer running)",
                    existing_pid
                );
                fs::remove_file(&self.lock_file)
                    .with_context(|| format!("Failed to remove stale lock file at {}", self.lock_file.display()))?;
            }
        }

        // Lock is available. Write a placeholder PID (the caller).
        // The real daemon PID should be written via write_daemon_pid()
        // after the daemon process is spawned.
        let current_pid = process::id();
        fs::write(&self.lock_file, current_pid.to_string())
            .with_context(|| format!("Failed to write lock file at {}", self.lock_file.display()))?;

        self.acquired = true;
        tracing::info!(pid = current_pid, "Acquired process lock (placeholder PID written)");
        Ok(true)
    }

    /// Write the actual daemon PID to the lock file.
    ///
    /// Call this after spawning the Lore daemon process to replace
    /// the placeholder PID with the real daemon PID.
    pub fn write_daemon_pid(&self, daemon_pid: u32) -> Result<()> {
        fs::write(&self.lock_file, daemon_pid.to_string())
            .context(format!(
                "Failed to write daemon PID {} to lock file at {}",
                daemon_pid,
                self.lock_file.display()
            ))?;
        tracing::info!(
            daemon_pid,
            lock_file = %self.lock_file.display(),
            "Wrote daemon PID to lock file"
        );
        Ok(())
    }

    /// Read the daemon PID from the lock file, if present.
    pub fn read_daemon_pid(&self) -> Result<Option<u32>> {
        if !self.lock_file.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&self.lock_file)
            .context(format!("Failed to read PID from lock file at {}", self.lock_file.display()))?;
        let pid = content.trim().parse::<u32>()
            .context(format!("Failed to parse PID '{}' from lock file at {}", content.trim(), self.lock_file.display()))?;
        Ok(Some(pid))
    }

    /// Release the lock
    pub fn release(&mut self) -> Result<()> {
        if self.acquired && self.lock_file.exists() {
            fs::remove_file(&self.lock_file)
                .context("Failed to remove lock file")?;
            self.acquired = false;
            tracing::info!("Released process lock");
        }
        Ok(())
    }

    /// Check if a process with the given PID is running
    #[cfg(unix)]
    fn is_process_running(&self, pid: u32) -> bool {
        // On Unix, send signal 0 to check if process exists
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        
        kill(Pid::from_raw(pid as i32), None).is_ok()
    }

    #[cfg(windows)]
    fn is_process_running(&self, pid: u32) -> bool {
        // On Windows, use OpenProcess to check if process exists
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

impl Drop for ProcessLock {
    fn drop(&mut self) {
        // Best-effort cleanup on drop
        let _ = self.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_process_lock_acquire_release() {
        let temp_dir = TempDir::new().unwrap();
        let mut lock = ProcessLock::new(temp_dir.path());

        // First acquire should succeed
        assert!(lock.try_acquire().unwrap());
        assert!(lock.acquired);

        // Release
        lock.release().unwrap();
        assert!(!lock.acquired);

        // Re-acquire should succeed
        assert!(lock.try_acquire().unwrap());
    }

    #[test]
    fn test_process_lock_double_acquire() {
        let temp_dir = TempDir::new().unwrap();
        let mut lock1 = ProcessLock::new(temp_dir.path());
        let mut lock2 = ProcessLock::new(temp_dir.path());

        // First acquire should succeed
        assert!(lock1.try_acquire().unwrap());

        // Second acquire should fail (already held by same PID — still "running")
        assert!(!lock2.try_acquire().unwrap());

        // Release first lock
        lock1.release().unwrap();

        // Now second acquire should succeed
        assert!(lock2.try_acquire().unwrap());
    }

    #[test]
    fn test_process_lock_cleanup_on_drop() {
        let temp_dir = TempDir::new().unwrap();
        let lock_file = temp_dir.path().join("lore").join("pid");

        {
            let mut lock = ProcessLock::new(temp_dir.path());
            lock.try_acquire().unwrap();
            assert!(lock_file.exists());
        }

        // Lock should be cleaned up on drop
        assert!(!lock_file.exists());
    }

    #[test]
    fn test_daemon_pid_write_and_read() {
        let temp_dir = TempDir::new().unwrap();
        let lock_file = temp_dir.path().join("lore").join("pid");

        // Create parent directory
        std::fs::create_dir_all(temp_dir.path().join("lore")).unwrap();

        let lock = ProcessLock::new(temp_dir.path());
        lock.write_daemon_pid(12345).unwrap();

        assert!(lock_file.exists());
        let content = std::fs::read_to_string(&lock_file).unwrap();
        assert_eq!(content, "12345");

        // Read it back
        let read_pid = lock.read_daemon_pid().unwrap();
        assert_eq!(read_pid, Some(12345));
    }

    #[test]
    fn test_read_daemon_pid_no_file() {
        let temp_dir = TempDir::new().unwrap();
        let lock = ProcessLock::new(temp_dir.path());
        let pid = lock.read_daemon_pid().unwrap();
        assert_eq!(pid, None);
    }

    #[test]
    fn test_stale_lock_removal() {
        let temp_dir = TempDir::new().unwrap();
        let lock_file = temp_dir.path().join("lore").join("pid");

        // Write a PID that is definitely not running (PID 1 is init on Unix,
        // but might be PID 0 on some systems — use a large number instead)
        std::fs::create_dir_all(lock_file.parent().unwrap()).unwrap();
        std::fs::write(&lock_file, "9999999").unwrap();

        let mut lock = ProcessLock::new(temp_dir.path());
        // Should detect stale lock and succeed
        assert!(lock.try_acquire().unwrap());
        // Lock file should now contain the current process PID
        let content = std::fs::read_to_string(&lock_file).unwrap();
        assert_eq!(content.trim().parse::<u32>().unwrap(), std::process::id());
    }
}
