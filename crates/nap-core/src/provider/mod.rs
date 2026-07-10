// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Provider architecture for Lore server backends
//!
//! This module provides the provider abstraction that allows NAP to work with
//! different Lore server deployments (Local, Portals Cloud, Remote) while
//! maintaining a consistent repository API.

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use async_trait::async_trait;

pub mod local;
pub mod portals_cloud;
pub mod remote;

use local::LocalProvider;
use portals_cloud::PortalsCloudProvider;
use remote::RemoteProvider;

/// Provider type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    Local,
    PortalsCloud,
    Remote,
}

/// Get default workspace ID from environment or use default
///
/// The workspace_id identifies a workspace within a Lore server instance.
/// It scopes repositories to a specific workspace, allowing multiple
/// isolated workspaces on the same server.
///
/// Environment variable: NAP_WORKSPACE_ID
/// Default: "default"
pub fn get_default_workspace_id() -> String {
    std::env::var("NAP_WORKSPACE_ID").unwrap_or_else(|_| "default".to_string())
}

/// Check if NAP debug mode is enabled
///
/// Debug mode provides verbose logging for troubleshooting and development.
/// When enabled, additional debug messages are logged throughout the SDK.
///
/// Environment variable: NAP_DEBUG
/// Values: "1", "true", "yes" (case-insensitive) to enable
pub fn is_debug_enabled() -> bool {
    if let Ok(debug_var) = std::env::var("NAP_DEBUG") {
        let debug_lower = debug_var.to_lowercase();
        debug_var == "1" || debug_lower == "true" || debug_lower == "yes"
    } else {
        false
    }
}

impl ProviderType {
    /// Parse provider type from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "local" => Ok(ProviderType::Local),
            "portals-cloud" | "portalscloud" => Ok(ProviderType::PortalsCloud),
            "remote" => Ok(ProviderType::Remote),
            _ => anyhow::bail!("Unknown provider type: {}", s),
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &str {
        match self {
            ProviderType::Local => "local",
            ProviderType::PortalsCloud => "portals-cloud",
            ProviderType::Remote => "remote",
        }
    }
}

/// Provider trait for Lore server backends
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    /// Get provider type
    fn provider_type(&self) -> ProviderType;

    /// Get provider name
    fn name(&self) -> &str;

    /// Initialize the provider
    async fn initialize(&self) -> Result<()>;

    /// Ensure the provider is ready for use
    async fn ensure_ready(&self) -> Result<()>;

    /// Get the Lore server URL base
    fn lore_url_base(&self) -> Result<String>;

    /// Get the workspace ID
    fn workspace_id(&self) -> &str;

    /// Check if provider is healthy
    async fn health_check(&self) -> Result<bool>;

    /// Get provider status
    async fn status(&self) -> Result<ProviderStatus>;
}

/// Provider status information
#[derive(Debug, Clone)]
pub struct ProviderStatus {
    pub provider_type: ProviderType,
    pub ready: bool,
    pub healthy: bool,
    pub url_base: String,
    pub workspace_id: String,
    pub message: String,
}

/// Provider factory for creating provider instances
pub struct ProviderFactory {
    nap_home: std::path::PathBuf,
}

impl ProviderFactory {
    /// Create a new provider factory
    pub fn new(nap_home: &Path) -> Self {
        Self {
            nap_home: nap_home.to_path_buf(),
        }
    }

    /// Create a provider by type
    pub fn create_provider(&self, provider_type: ProviderType) -> Result<Arc<dyn Provider>> {
        match provider_type {
            ProviderType::Local => {
                let provider: LocalProvider = LocalProvider::new(&self.nap_home);
                Ok(Arc::new(provider) as Arc<dyn Provider>)
            }
            ProviderType::PortalsCloud => {
                let provider: PortalsCloudProvider = PortalsCloudProvider::new();
                Ok(Arc::new(provider) as Arc<dyn Provider>)
            }
            ProviderType::Remote => {
                anyhow::bail!("Remote provider requires configuration (URL, workspace)");
            }
        }
    }

    /// Create a remote provider with configuration
    pub fn create_remote_provider(
        &self,
        url_base: &str,
        workspace_id: &str,
    ) -> Result<Arc<dyn Provider>> {
        let provider: RemoteProvider = RemoteProvider::new(url_base, workspace_id);
        Ok(Arc::new(provider) as Arc<dyn Provider>)
    }
}

/// Provider manager for managing the active provider
pub struct ProviderManager {
    nap_home: std::path::PathBuf,
    active_provider: Option<Arc<dyn Provider>>,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new(nap_home: &Path) -> Self {
        Self {
            nap_home: nap_home.to_path_buf(),
            active_provider: None,
        }
    }

    /// Load configured provider from disk
    pub fn load_configured_provider(&mut self) -> Result<Option<Arc<dyn Provider>>> {
        let config_path = self.nap_home.join("provider.toml");

        if !config_path.exists() {
            tracing::debug!("No provider configuration found at {}", config_path.display());
            if is_debug_enabled() {
                tracing::debug!("NAP debug mode: Provider config path does not exist: {}", config_path.display());
            }
            return Ok(None);
        }

        let config_content = std::fs::read_to_string(&config_path)
            .context(format!(
                "Failed to read provider configuration from '{}'",
                config_path.display()
            ))?;

        if is_debug_enabled() {
            tracing::debug!("NAP debug mode: Loaded provider config from {}", config_path.display());
            // Limit config content logging to avoid performance issues with large configs
            let config_preview = if config_content.len() > 500 {
                format!("{}... (truncated, {} total chars)", &config_content[..500], config_content.len())
            } else {
                config_content.clone()
            };
            tracing::debug!("NAP debug mode: Config content: {}", config_preview);
        }
        
        let config: ProviderConfig = toml::from_str(&config_content)
            .context(format!(
                "Failed to parse provider configuration from '{}'. \
                 The file may be corrupted. Delete it and run 'nap init' to reconfigure.",
                config_path.display()
            ))?;

        // Validate the configuration
        config.validate()
            .context(format!(
                "Invalid provider configuration in '{}'",
                config_path.display()
            ))?;

        let factory = ProviderFactory::new(&self.nap_home);
        
        let provider = match config.provider_type.as_str() {
            "local" => Some(factory.create_provider(ProviderType::Local)?),
            "portals-cloud" => Some(factory.create_provider(ProviderType::PortalsCloud)?),
            "remote" => {
                let url_base = config.remote_url
                    .context("Remote provider requires remote_url in provider.toml")?;
                let workspace_id = config.workspace_id
                    .context("Remote provider requires workspace_id in provider.toml")?;
                Some(factory.create_remote_provider(&url_base, &workspace_id)?)
            }
            _ => unreachable!("validated above"),
        };

        if let Some(ref provider) = provider {
            self.active_provider = Some(provider.clone());
            if is_debug_enabled() {
                tracing::debug!("NAP debug mode: Loaded provider: {}", provider.name());
                tracing::debug!("NAP debug mode: Provider type: {:?}", provider.provider_type());
            }
            tracing::info!(
                provider = %provider.name(),
                provider_type = %config.provider_type,
                "Loaded provider configuration"
            );
        }

        Ok(provider)
    }

    /// Set the active provider
    pub fn set_active_provider(&mut self, provider: Arc<dyn Provider>) {
        self.active_provider = Some(provider);
    }

    /// Get the active provider
    pub fn active_provider(&self) -> Option<&Arc<dyn Provider>> {
        self.active_provider.as_ref()
    }

    /// Save provider configuration to disk
    pub fn save_provider_config(&self, provider: &dyn Provider) -> Result<()> {
        let config = ProviderConfig {
            provider_type: provider.provider_type().as_str().to_string(),
            remote_url: provider.lore_url_base().ok(),
            workspace_id: Some(provider.workspace_id().to_string()),
        };

        let config_content = toml::to_string_pretty(&config)
            .context("Failed to serialize provider configuration")?;

        let config_path = self.nap_home.join("provider.toml");
        std::fs::write(&config_path, config_content)
            .context("Failed to write provider configuration")?;

        Ok(())
    }

    /// Ensure the active provider is ready
    pub async fn ensure_provider_ready(&self) -> Result<()> {
        if let Some(provider) = &self.active_provider {
            provider.ensure_ready().await?;
            Ok(())
        } else {
            anyhow::bail!("No active provider configured");
        }
    }
}

/// Provider configuration stored on disk
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ProviderConfig {
    provider_type: String,
    remote_url: Option<String>,
    workspace_id: Option<String>,
}

impl ProviderConfig {
    /// Validate the provider configuration.
    ///
    /// Ensures the config is complete and consistent for the declared provider type.
    /// Uses manual validation with descriptive error messages for better UX than schema validation.
    fn validate(&self) -> Result<()> {
        if is_debug_enabled() {
            tracing::debug!("NAP debug mode: Validating provider config for type: {}", self.provider_type);
        }
        // Validate provider_type is known
        let provider_type = ProviderType::from_str(&self.provider_type)
            .context(format!(
                "Invalid provider_type '{}' in provider.toml. \
                 Expected one of: 'local', 'portals-cloud', 'remote'. \
                 Fix the file at the NAP home directory or run 'nap init' to reconfigure.",
                self.provider_type
            ))?;

        // Validate type-specific required fields
        match provider_type {
            ProviderType::Remote => {
                if self.remote_url.is_none() {
                    anyhow::bail!(
                        "Remote provider requires 'remote_url' in provider.toml. \
                         Add remote_url = \"lore://host:port\" to the [provider] section, \
                         or reconfigure with 'nap init'."
                    );
                }
                if self.workspace_id.is_none() {
                    anyhow::bail!(
                        "Remote provider requires 'workspace_id' in provider.toml. \
                         Add workspace_id = \"your-workspace\" to the [provider] section, \
                         or reconfigure with 'nap init'."
                    );
                }
                // Validate URL format
                let url = self.remote_url.as_ref().unwrap();
                if !url.starts_with("lore://") && !url.starts_with("lores://") {
                    anyhow::bail!(
                        "Invalid remote_url '{}' in provider.toml. \
                         URL must start with 'lore://' or 'lores://'. \
                         Example: lore://localhost:41337",
                        url
                    );
                }
            }
            ProviderType::Local => {
                // Local provider has no required config fields beyond provider_type
                tracing::debug!("Local provider config validated (no additional fields required)");
            }
            ProviderType::PortalsCloud => {
                // Cloud provider reads auth from environment variables
                tracing::debug!("Portals Cloud provider config validated (auth from environment)");
            }
        }

        if is_debug_enabled() {
            tracing::debug!("NAP debug mode: Provider config validation successful for type: {}", self.provider_type);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_provider_type_from_str() {
        assert_eq!(ProviderType::from_str("local").unwrap(), ProviderType::Local);
        assert_eq!(ProviderType::from_str("portals-cloud").unwrap(), ProviderType::PortalsCloud);
        assert_eq!(ProviderType::from_str("portalscloud").unwrap(), ProviderType::PortalsCloud);
        assert_eq!(ProviderType::from_str("remote").unwrap(), ProviderType::Remote);
    }

    #[test]
    fn test_provider_type_as_str() {
        assert_eq!(ProviderType::Local.as_str(), "local");
        assert_eq!(ProviderType::PortalsCloud.as_str(), "portals-cloud");
        assert_eq!(ProviderType::Remote.as_str(), "remote");
    }

    #[test]
    fn test_provider_type_from_str_unknown() {
        let result = ProviderType::from_str("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown provider type"));
    }

    #[test]
    fn test_provider_config_validation_local() {
        let config = ProviderConfig {
            provider_type: "local".to_string(),
            remote_url: None,
            workspace_id: None,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_validation_portals_cloud() {
        let config = ProviderConfig {
            provider_type: "portals-cloud".to_string(),
            remote_url: None,
            workspace_id: None,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_validation_remote_missing_url() {
        let config = ProviderConfig {
            provider_type: "remote".to_string(),
            remote_url: None,
            workspace_id: Some("default".to_string()),
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("remote_url"));
    }

    #[test]
    fn test_provider_config_validation_remote_missing_workspace() {
        let config = ProviderConfig {
            provider_type: "remote".to_string(),
            remote_url: Some("lore://localhost:41337".to_string()),
            workspace_id: None,
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("workspace_id"));
    }

    #[test]
    fn test_provider_config_validation_remote_bad_url() {
        let config = ProviderConfig {
            provider_type: "remote".to_string(),
            remote_url: Some("http://localhost:41337".to_string()),
            workspace_id: Some("default".to_string()),
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("lore://"));
    }

    #[test]
    fn test_provider_config_validation_remote_valid() {
        let config = ProviderConfig {
            provider_type: "remote".to_string(),
            remote_url: Some("lore://localhost:41337".to_string()),
            workspace_id: Some("default".to_string()),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_validation_unknown_type() {
        let config = ProviderConfig {
            provider_type: "nonexistent".to_string(),
            remote_url: None,
            workspace_id: None,
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid provider_type"));
    }

    #[test]
    fn test_provider_factory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let factory = ProviderFactory::new(temp_dir.path());
        
        let local = factory.create_provider(ProviderType::Local).unwrap();
        assert_eq!(local.provider_type(), ProviderType::Local);
        
        let cloud = factory.create_provider(ProviderType::PortalsCloud).unwrap();
        assert_eq!(cloud.provider_type(), ProviderType::PortalsCloud);
        
        // Remote without config should fail
        let remote = factory.create_provider(ProviderType::Remote);
        assert!(remote.is_err());
    }

    #[test]
    fn test_provider_factory_remote_with_config() {
        let temp_dir = TempDir::new().unwrap();
        let factory = ProviderFactory::new(temp_dir.path());
        
        let remote = factory.create_remote_provider("lore://localhost:41337", "test-ws").unwrap();
        assert_eq!(remote.provider_type(), ProviderType::Remote);
        assert_eq!(remote.workspace_id(), "test-ws");
    }

    #[test]
    fn test_provider_manager_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create and save a provider config
        let mut manager = ProviderManager::new(temp_dir.path());
        let factory = ProviderFactory::new(temp_dir.path());
        let local_provider = factory.create_provider(ProviderType::Local).unwrap();
        
        manager.set_active_provider(local_provider.clone());
        manager.save_provider_config(local_provider.as_ref()).unwrap();
        
        // Verify config file was written
        let config_path = temp_dir.path().join("provider.toml");
        assert!(config_path.exists());
        
        // Load it back
        let mut manager2 = ProviderManager::new(temp_dir.path());
        let loaded = manager2.load_configured_provider().unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().provider_type(), ProviderType::Local);
    }

    #[test]
    fn test_provider_config_roundtrip_serialization() {
        let config = ProviderConfig {
            provider_type: "remote".to_string(),
            remote_url: Some("lore://host:41337".to_string()),
            workspace_id: Some("my-workspace".to_string()),
        };
        
        let serialized = toml::to_string_pretty(&config).unwrap();
        assert!(serialized.contains("remote"));
        assert!(serialized.contains("lore://host:41337"));
        
        let deserialized: ProviderConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.provider_type, "remote");
        assert_eq!(deserialized.remote_url, Some("lore://host:41337".to_string()));
        assert_eq!(deserialized.workspace_id, Some("my-workspace".to_string()));
    }
}
