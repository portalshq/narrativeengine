// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Hosted Lore provider. Repository authentication belongs to Lore's shared login.
use super::{Provider, ProviderStatus, ProviderType};
use anyhow::Result;

pub const PORTALS_CLOUD_URL: &str = "grpcs://lore.portals.works";

pub struct PortalsCloudProvider {
    remote: super::remote::RemoteProvider,
}

impl Default for PortalsCloudProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl PortalsCloudProvider {
    pub fn new() -> Self {
        Self {
            remote: super::remote::RemoteProvider::new_with_default_workspace(PORTALS_CLOUD_URL),
        }
    }
    pub fn with_workspace_id(mut self, workspace_id: &str) -> Self {
        self.remote = super::remote::RemoteProvider::new(PORTALS_CLOUD_URL, workspace_id);
        self
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
        Ok(())
    }
    async fn ensure_ready(&self) -> Result<()> {
        self.remote.ensure_ready().await
    }
    fn lore_url_base(&self) -> Result<String> {
        Ok(PORTALS_CLOUD_URL.into())
    }
    fn workspace_id(&self) -> &str {
        self.remote.workspace_id()
    }
    async fn health_check(&self) -> Result<bool> {
        self.remote.health_check().await
    }
    async fn status(&self) -> Result<ProviderStatus> {
        let healthy = self.health_check().await?;
        Ok(ProviderStatus {
            provider_type: self.provider_type(),
            ready: healthy,
            healthy,
            url_base: self.lore_url_base()?,
            workspace_id: self.workspace_id().into(),
            message: if healthy {
                "Connected"
            } else {
                "Server unreachable"
            }
            .into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cloud_endpoint_needs_no_legacy_account_environment() {
        let provider = PortalsCloudProvider::new().with_workspace_id("workspace");
        assert_eq!(provider.lore_url_base().unwrap(), PORTALS_CLOUD_URL);
        assert_eq!(provider.workspace_id(), "workspace");
        assert_eq!(provider.provider_type(), ProviderType::PortalsCloud);
    }
}
