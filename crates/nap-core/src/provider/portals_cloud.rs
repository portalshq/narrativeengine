// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Portals Cloud provider for Lore server
//!
//! Manages repository operations on the Portals Cloud hosted Lore service.

use anyhow::{Context, Result};
use tracing::{info, warn};
use async_trait::async_trait;

use super::{Provider, ProviderStatus, ProviderType};

/// Portals Cloud provider for hosted Lore service
pub struct PortalsCloudProvider {
    api_url: String,
    workspace_id: String,
    account_id: Option<String>,
    auth_token: Option<String>,
}

impl PortalsCloudProvider {
    /// Create a new Portals Cloud provider
    pub fn new() -> Self {
        Self {
            api_url: "https://api.portals.sh".to_string(),
            workspace_id: super::get_default_workspace_id(),
            account_id: std::env::var("NAP_PORTALS_ACCOUNT_ID").ok(),
            auth_token: std::env::var("NAP_PORTALS_AUTH_TOKEN").ok(),
        }
    }

    /// Set custom API URL
    pub fn with_api_url(mut self, url: &str) -> Self {
        self.api_url = url.to_string();
        self
    }

    /// Set custom workspace ID
    pub fn with_workspace_id(mut self, workspace_id: &str) -> Self {
        self.workspace_id = workspace_id.to_string();
        self
    }

    /// Set account ID
    pub fn with_account_id(mut self, account_id: &str) -> Self {
        self.account_id = Some(account_id.to_string());
        self
    }

    /// Set auth token
    pub fn with_auth_token(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }

    /// Check if authenticated
    fn is_authenticated(&self) -> bool {
        self.account_id.is_some() && self.auth_token.is_some()
    }

    /// Get Lore server URL from Portals Cloud
    fn resolve_lore_url(&self) -> Result<String> {
        if !self.is_authenticated() {
            anyhow::bail!("Portals Cloud provider requires authentication (NAP_PORTALS_ACCOUNT_ID and NAP_PORTALS_AUTH_TOKEN)");
        }

        // In a real implementation, this would query the Portals Cloud API
        // to get the Lore server URL for the account/workspace
        // For now, return a placeholder
        Ok(format!("lore://{}.portals.sh", self.account_id.as_ref().unwrap()))
    }
}

#[async_trait::async_trait]
impl Provider for PortalsCloudProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::PortalsCloud
    }

    fn name(&self) -> &str {
        "Portals Cloud"
    }

    async fn initialize(&self) -> Result<()> {
        info!("Initializing Portals Cloud provider");

        if !self.is_authenticated() {
            warn!("Portals Cloud provider not authenticated");
        }

        info!("Portals Cloud provider initialized");
        Ok(())
    }

    async fn ensure_ready(&self) -> Result<()> {
        info!("Ensuring Portals Cloud provider is ready");

        self.initialize().await?;

        if !self.is_authenticated() {
            anyhow::bail!("Portals Cloud provider requires authentication");
        }

        // Check connectivity to Portals Cloud API
        let health_url = format!("{}/health", self.api_url);
        let response = reqwest::get(&health_url).await
            .context("Failed to connect to Portals Cloud API")?;

        if !response.status().is_success() {
            anyhow::bail!("Portals Cloud API health check failed: {}", response.status());
        }

        info!("Portals Cloud provider is ready");
        Ok(())
    }

    fn lore_url_base(&self) -> Result<String> {
        self.resolve_lore_url()
    }

    fn workspace_id(&self) -> &str {
        &self.workspace_id
    }

    async fn health_check(&self) -> Result<bool> {
        if !self.is_authenticated() {
            return Ok(false);
        }

        let health_url = format!("{}/health", self.api_url);
        match reqwest::get(&health_url).await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn status(&self) -> Result<ProviderStatus> {
        let authenticated = self.is_authenticated();
        let healthy = self.health_check().await.unwrap_or(false);
        let url_base = if authenticated {
            self.lore_url_base().unwrap_or_else(|_| "unknown".to_string())
        } else {
            "not authenticated".to_string()
        };

        let message = if !authenticated {
            "Not authenticated".to_string()
        } else if !healthy {
            "API unreachable".to_string()
        } else {
            "Connected".to_string()
        };

        Ok(ProviderStatus {
            provider_type: self.provider_type(),
            ready: authenticated && healthy,
            healthy,
            url_base,
            workspace_id: self.workspace_id.clone(),
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portals_cloud_provider_creation() {
        let provider = PortalsCloudProvider::new();
        assert_eq!(provider.provider_type(), ProviderType::PortalsCloud);
        assert_eq!(provider.name(), "Portals Cloud");
        assert_eq!(provider.workspace_id(), "default");
    }

    #[test]
    fn test_portals_cloud_provider_custom_config() {
        let provider = PortalsCloudProvider::new()
            .with_api_url("https://custom.api.com")
            .with_workspace_id("custom-workspace")
            .with_account_id("account-123")
            .with_auth_token("token-456");

        assert_eq!(provider.api_url, "https://custom.api.com");
        assert_eq!(provider.workspace_id(), "custom-workspace");
        assert_eq!(provider.account_id, Some("account-123".to_string()));
        assert_eq!(provider.auth_token, Some("token-456".to_string()));
    }

    #[test]
    fn test_authentication_check() {
        let provider = PortalsCloudProvider::new();
        assert!(!provider.is_authenticated());

        let provider = PortalsCloudProvider::new()
            .with_account_id("account-123")
            .with_auth_token("token-456");
        assert!(provider.is_authenticated());
    }
}
