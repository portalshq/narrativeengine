// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Remote provider for Lore server
//!
//! Manages repository operations on a remote Lore instance.

use anyhow::{Context, Result};
use tracing::info;

use super::{Provider, ProviderStatus, ProviderType};

/// Remote provider for custom Lore server
pub struct RemoteProvider {
    url_base: String,
    workspace_id: String,
    auth_token: Option<String>,
}

impl RemoteProvider {
    /// Create a new remote provider
    pub fn new(url_base: &str, workspace_id: &str) -> Self {
        Self {
            url_base: url_base.to_string(),
            workspace_id: workspace_id.to_string(),
            auth_token: std::env::var("NAP_REMOTE_AUTH_TOKEN").ok(),
        }
    }

    /// Create a new remote provider with default workspace ID
    pub fn new_with_default_workspace(url_base: &str) -> Self {
        Self {
            url_base: url_base.to_string(),
            workspace_id: super::get_default_workspace_id(),
            auth_token: std::env::var("NAP_REMOTE_AUTH_TOKEN").ok(),
        }
    }

    /// Set custom auth token
    pub fn with_auth_token(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }

    /// Parse URL to extract HTTP health check endpoint
    /// 
    /// Lore server uses port 41337 for gRPC/QUIC (lore:// URLs) and port 41339 for HTTP.
    /// This function converts lore://host:41337 to http://host:41339/health_check
    fn http_health_url(&self) -> Result<String> {
        // Parse the lore:// URL to extract host and optionally port
        let (scheme, rest) = if self.url_base.starts_with("lore://") {
            ("http", &self.url_base[7..]) // "lore://" is 7 characters
        } else if self.url_base.starts_with("lores://") {
            ("https", &self.url_base[8..]) // "lores://" is 8 characters
        } else {
            anyhow::bail!("Invalid Lore URL format: {}", self.url_base);
        };

        // Split host and port if present
        let host = if rest.contains(':') {
            // Extract host part, ignore the lore port (typically 41337)
            rest.split(':').next().unwrap()
        } else {
            rest
        };

        // Lore server uses port 41339 for HTTP health checks
        let http_port = 41339;
        
        Ok(format!("{}://{}:{}/health_check", scheme, host, http_port))
    }
}

#[async_trait::async_trait]
impl Provider for RemoteProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Remote
    }

    fn name(&self) -> &str {
        "Remote Lore Server"
    }

    async fn initialize(&self) -> Result<()> {
        info!("Initializing Remote provider for {}", self.url_base);
        info!("Remote provider initialized");
        Ok(())
    }

    async fn ensure_ready(&self) -> Result<()> {
        info!("Ensuring Remote provider is ready");

        self.initialize().await?;

        // Check connectivity to remote server
        let health_url = self.http_health_url()?;
        let response = reqwest::get(&health_url)
            .await
            .context("Failed to connect to remote Lore server")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Remote Lore server health check failed: {}",
                response.status()
            );
        }

        info!("Remote provider is ready");
        Ok(())
    }

    fn lore_url_base(&self) -> Result<String> {
        Ok(self.url_base.clone())
    }

    fn workspace_id(&self) -> &str {
        &self.workspace_id
    }

    async fn health_check(&self) -> Result<bool> {
        let health_url = self.http_health_url()?;
        match reqwest::get(&health_url).await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn status(&self) -> Result<ProviderStatus> {
        let healthy = self.health_check().await.unwrap_or(false);

        let message = if healthy {
            "Connected".to_string()
        } else {
            "Server unreachable".to_string()
        };

        Ok(ProviderStatus {
            provider_type: self.provider_type(),
            ready: healthy,
            healthy,
            url_base: self.url_base.clone(),
            workspace_id: self.workspace_id.clone(),
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_provider_creation() {
        let provider = RemoteProvider::new("lore://localhost:41337", "default");
        assert_eq!(provider.provider_type(), ProviderType::Remote);
        assert_eq!(provider.name(), "Remote Lore Server");
        assert_eq!(provider.workspace_id(), "default");
        assert_eq!(provider.url_base, "lore://localhost:41337");
    }

    #[test]
    fn test_remote_provider_custom_auth() {
        let provider = RemoteProvider::new("lore://localhost:41337", "default")
            .with_auth_token("custom-token");
        assert_eq!(provider.auth_token, Some("custom-token".to_string()));
    }

    #[test]
    fn test_http_health_url() {
        let provider = RemoteProvider::new("lore://localhost:41337", "default");
        assert_eq!(
            provider.http_health_url().unwrap(),
            "http://localhost:41339/health_check"
        );

        let provider = RemoteProvider::new("lores://example.com:41337", "default");
        assert_eq!(
            provider.http_health_url().unwrap(),
            "https://example.com:41339/health_check"
        );
        
        // Test without port in URL
        let provider = RemoteProvider::new("lore://192.168.0.27", "default");
        assert_eq!(
            provider.http_health_url().unwrap(),
            "http://192.168.0.27:41339/health_check"
        );
    }
}
