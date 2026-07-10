// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Local provider for Lore server
//!
//! Manages a local Lore daemon for repository operations.

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

use super::{Provider, ProviderStatus, ProviderType};
use crate::server::ServerManager;

/// Local provider that manages a local Lore daemon
pub struct LocalProvider {
    server_manager: Arc<ServerManager>,
    workspace_id: String,
}

impl LocalProvider {
    /// Create a new local provider
    pub fn new(nap_home: &Path) -> Self {
        let server_manager = Arc::new(ServerManager::new(nap_home));
        Self {
            server_manager,
            workspace_id: super::get_default_workspace_id(),
        }
    }

    /// Set custom workspace ID
    pub fn with_workspace_id(mut self, workspace_id: &str) -> Self {
        self.workspace_id = workspace_id.to_string();
        self
    }
}

#[async_trait::async_trait]
impl Provider for LocalProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Local
    }

    fn name(&self) -> &str {
        "Local Lore Daemon"
    }

    async fn initialize(&self) -> Result<()> {
        info!("Initializing Local provider");

        // Ensure Lore is installed
        self.server_manager.ensure_installed()?;

        // Ensure Lore is configured
        self.server_manager.ensure_configured()?;

        info!("Local provider initialized");
        Ok(())
    }

    async fn ensure_ready(&self) -> Result<()> {
        info!("Ensuring Local provider is ready");

        // Initialize if needed
        self.initialize().await?;

        // Ensure server is running
        self.server_manager.ensure_running().await?;

        info!("Local provider is ready");
        Ok(())
    }

    fn lore_url_base(&self) -> Result<String> {
        Ok("lore://localhost:41337".to_string())
    }

    fn workspace_id(&self) -> &str {
        &self.workspace_id
    }

    async fn health_check(&self) -> Result<bool> {
        self.server_manager.health_check().await
    }

    async fn status(&self) -> Result<ProviderStatus> {
        let server_status = self.server_manager.status().await?;
        let healthy = self.health_check().await.unwrap_or(false);

        Ok(ProviderStatus {
            provider_type: self.provider_type(),
            ready: server_status.is_ready(),
            healthy,
            url_base: self.lore_url_base()?,
            workspace_id: self.workspace_id.clone(),
            message: server_status.status_message(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_local_provider_creation() {
        let temp_dir = TempDir::new().unwrap();
        let provider = LocalProvider::new(temp_dir.path());
        assert_eq!(provider.provider_type(), ProviderType::Local);
        assert_eq!(provider.name(), "Local Lore Daemon");
        assert_eq!(provider.workspace_id(), "default");
    }

    #[test]
    fn test_local_provider_custom_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let provider = LocalProvider::new(temp_dir.path()).with_workspace_id("custom-workspace");
        assert_eq!(provider.workspace_id(), "custom-workspace");
    }
}
