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
    http_url: Option<String>,
}

impl RemoteProvider {
    /// Create a new remote provider
    pub fn new(url_base: &str, workspace_id: &str) -> Self {
        Self {
            url_base: url_base.to_string(),
            workspace_id: workspace_id.to_string(),
            auth_token: std::env::var("NAP_REMOTE_AUTH_TOKEN").ok(),
            http_url: None,
        }
    }

    /// Create a new remote provider with default workspace ID
    pub fn new_with_default_workspace(url_base: &str) -> Self {
        Self {
            url_base: url_base.to_string(),
            workspace_id: super::get_default_workspace_id(),
            auth_token: std::env::var("NAP_REMOTE_AUTH_TOKEN").ok(),
            http_url: None,
        }
    }

    pub fn with_http_url(mut self, url: &str) -> Result<Self> {
        super::http::validate_origin(url)?;
        self.http_url = Some(url.trim_end_matches('/').to_string());
        Ok(self)
    }

    /// Set custom auth token
    pub fn with_auth_token(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }

    async fn probe(&self) -> Result<()> {
        let rpc = reqwest::Url::parse(&self.url_base)?;
        let tls_edge = rpc.host_str() == Some("lore.portals.works")
            || (matches!(rpc.scheme(), "grpcs" | "lores" | "https")
                && matches!(rpc.port(), None | Some(443)));
        if tls_edge {
            // TLS edges may expose only selected HTTP routes, while gRPC remains
            // available on the origin. Connectivity does not require repository auth.
            tonic::transport::Endpoint::from_shared(super::http::default_origin(&self.url_base)?)?
                .connect_timeout(std::time::Duration::from_secs(10))
                .connect()
                .await
                .context("Failed to connect to Lore TLS endpoint")?;
        } else {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()?
                .get(self.http_health_url()?)
                .send()
                .await
                .context("Failed to connect to remote Lore server")?
                .error_for_status()
                .context("Remote Lore server health check failed")?;
        }
        Ok(())
    }

    /// Parse URL to extract HTTP health check endpoint
    ///
    /// Lore server uses port 41337 for gRPC/QUIC (lore:// URLs) and port 41339 for HTTP.
    /// This function converts lore://host:41337 to http://host:41339/health_check
    fn http_health_url(&self) -> Result<String> {
        Ok(format!(
            "{}/health_check",
            self.http_url
                .clone()
                .map(Ok)
                .unwrap_or_else(|| super::http::default_origin(&self.url_base))?
        ))
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

        self.probe().await?;

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
        Ok(self.probe().await.is_ok())
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
