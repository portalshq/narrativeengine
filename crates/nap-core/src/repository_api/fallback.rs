// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Provider fallback UX
//!
//! Handles graceful fallback from cloud providers to local provider when
//! cloud services are unavailable.

use anyhow::{Context, Result};
use tracing::{info, warn};

use super::RepositoryApi;
use crate::provider::{ProviderFactory, ProviderType};

/// Fallback strategy for provider failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallbackStrategy {
    /// No fallback - fail on provider unavailability
    None,
    /// Automatically fallback to local provider
    Auto,
    /// Prompt user before falling back
    Prompt,
}

/// Fallback result
#[derive(Debug, Clone)]
pub enum FallbackResult {
    /// No fallback needed
    NotNeeded,
    /// Fallback successful
    Success {
        original_provider: ProviderType,
        fallback_provider: ProviderType,
    },
    /// Fallback failed
    Failed {
        original_provider: ProviderType,
        error: String,
    },
    /// Fallback declined by user
    Declined { original_provider: ProviderType },
}

/// Provider fallback handler
pub struct FallbackHandler {
    strategy: FallbackStrategy,
    nap_home: std::path::PathBuf,
}

impl FallbackHandler {
    /// Create a new fallback handler
    pub fn new(nap_home: &std::path::Path) -> Self {
        Self {
            strategy: FallbackStrategy::Prompt,
            nap_home: nap_home.to_path_buf(),
        }
    }

    /// Set fallback strategy
    pub fn with_strategy(mut self, strategy: FallbackStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Handle provider failure with fallback
    pub async fn handle_provider_failure(
        &self,
        repository_api: &mut RepositoryApi,
        original_provider: ProviderType,
        error: &str,
    ) -> Result<FallbackResult> {
        warn!(
            "Provider failure detected: {} - {}",
            original_provider.as_str(),
            error
        );

        // Only fallback from cloud providers to local
        if !matches!(
            original_provider,
            ProviderType::PortalsCloud | ProviderType::Remote
        ) {
            return Ok(FallbackResult::Failed {
                original_provider,
                error: format!(
                    "Cannot fallback from {} provider",
                    original_provider.as_str()
                ),
            });
        }

        match self.strategy {
            FallbackStrategy::None => Ok(FallbackResult::Failed {
                original_provider,
                error: "Fallback disabled".to_string(),
            }),
            FallbackStrategy::Auto => {
                self.perform_fallback(repository_api, original_provider)
                    .await
            }
            FallbackStrategy::Prompt => {
                // In a real implementation, this would prompt the user
                // For now, we'll simulate a prompt and default to yes
                info!("Prompting user for fallback to local provider");
                self.perform_fallback(repository_api, original_provider)
                    .await
            }
        }
    }

    /// Perform the actual fallback to local provider
    async fn perform_fallback(
        &self,
        repository_api: &mut RepositoryApi,
        original_provider: ProviderType,
    ) -> Result<FallbackResult> {
        info!("Attempting fallback to local provider");

        let factory = ProviderFactory::new(&self.nap_home);
        let local_provider = factory
            .create_provider(ProviderType::Local)
            .context("Failed to create local provider for fallback")?;

        // Initialize local provider
        local_provider
            .initialize()
            .await
            .context("Failed to initialize local provider during fallback")?;

        // Ensure local provider is ready
        local_provider
            .ensure_ready()
            .await
            .context("Failed to ensure local provider ready during fallback")?;

        // Update repository API with local provider
        repository_api
            .provider_manager_mut()
            .set_active_provider(local_provider.clone());

        // Save new provider configuration
        repository_api
            .provider_manager_mut()
            .save_provider_config(local_provider.as_ref())?;

        info!("Fallback to local provider successful");

        Ok(FallbackResult::Success {
            original_provider,
            fallback_provider: ProviderType::Local,
        })
    }

    /// Check if fallback should be offered based on error type
    pub fn should_offer_fallback(&self, error: &str) -> bool {
        // Offer fallback for network/connectivity errors
        error.to_lowercase().contains("unavailable")
            || error.to_lowercase().contains("timeout")
            || error.to_lowercase().contains("connection")
            || error.to_lowercase().contains("network")
    }

    /// Get user-friendly fallback message
    pub fn fallback_message(&self, original_provider: ProviderType) -> String {
        match original_provider {
            ProviderType::PortalsCloud => "Portals Cloud is currently unavailable.\n\
                 Start a local Lore server instead?\n\
                 Changes will remain local until synchronization.\n\
                 [Y/n]"
                .to_string(),
            ProviderType::Remote => "Remote Lore server is currently unavailable.\n\
                 Start a local Lore server instead?\n\
                 Changes will remain local until synchronization.\n\
                 [Y/n]"
                .to_string(),
            ProviderType::Local => "Local provider failed. No fallback available.".to_string(),
        }
    }
}

/// Extension trait for RepositoryApi to add fallback support
pub trait RepositoryApiFallback {
    /// Perform operation with automatic fallback.
    ///
    /// Executes the closure; if it fails with a network-related error and
    /// the current provider is cloud/remote, offers fallback to local.
    /// On successful fallback, retries the operation once.
    fn with_fallback<F, Fut, T>(
        &mut self,
        operation: F,
    ) -> impl std::future::Future<Output = Result<T>> + Send
    where
        F: FnMut(&mut Self) -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send + 'static;

    /// Check provider health and fallback if needed.
    /// Returns `Ok(true)` if provider is healthy or fallback succeeded.
    fn ensure_provider_with_fallback(
        &mut self,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;
}

impl RepositoryApiFallback for RepositoryApi {
    async fn with_fallback<F, Fut, T>(&mut self, mut operation: F) -> Result<T>
    where
        F: FnMut(&mut Self) -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send + 'static,
    {
        // Try the operation
        let result = operation(self).await;

        if let Err(e) = &result {
            // Check if we should offer fallback
            if let Some(provider) = self.active_provider() {
                let handler = FallbackHandler::new(&self.nap_home);
                let provider_type = provider.provider_type();

                if handler.should_offer_fallback(&e.to_string()) {
                    let fallback_result = handler
                        .handle_provider_failure(self, provider_type, &e.to_string())
                        .await?;

                    match fallback_result {
                        FallbackResult::Success { .. } => {
                            // Retry operation with fallback provider
                            operation(self).await
                        }
                        FallbackResult::Declined { .. } => result,
                        FallbackResult::Failed { error, .. } => Err(anyhow::anyhow!(error)),
                        FallbackResult::NotNeeded => result,
                    }
                } else {
                    result
                }
            } else {
                result
            }
        } else {
            result
        }
    }

    async fn ensure_provider_with_fallback(&mut self) -> Result<bool> {
        let handler = FallbackHandler::new(&self.nap_home);

        if let Some(provider) = self.active_provider() {
            let provider_type = provider.provider_type();

            // Check if provider is healthy
            let is_healthy = provider.health_check().await.unwrap_or(false);

            if is_healthy {
                Ok(true)
            } else {
                // Provider is unhealthy, try fallback
                let fallback_result = handler
                    .handle_provider_failure(self, provider_type, "Provider health check failed")
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;

                match fallback_result {
                    FallbackResult::Success { .. } => Ok(true),
                    FallbackResult::Declined { .. } => Ok(false),
                    FallbackResult::Failed { error, .. } => Err(anyhow::anyhow!(error)),
                    FallbackResult::NotNeeded => Ok(true),
                }
            }
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_fallback_handler_creation() {
        let temp_dir = TempDir::new().unwrap();
        let handler = FallbackHandler::new(temp_dir.path());
        assert_eq!(handler.strategy, FallbackStrategy::Prompt);
    }

    #[test]
    fn test_fallback_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let handler = FallbackHandler::new(temp_dir.path()).with_strategy(FallbackStrategy::Auto);
        assert_eq!(handler.strategy, FallbackStrategy::Auto);
    }

    #[test]
    fn test_should_offer_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let handler = FallbackHandler::new(temp_dir.path());

        assert!(handler.should_offer_fallback("Service unavailable"));
        assert!(handler.should_offer_fallback("Connection timeout"));
        assert!(handler.should_offer_fallback("Network error"));
        assert!(!handler.should_offer_fallback("Permission denied"));
    }

    #[test]
    fn test_fallback_message() {
        let temp_dir = TempDir::new().unwrap();
        let handler = FallbackHandler::new(temp_dir.path());

        let message = handler.fallback_message(ProviderType::PortalsCloud);
        assert!(message.contains("Portals Cloud is currently unavailable"));
        assert!(message.contains("Start a local Lore server instead"));

        let message = handler.fallback_message(ProviderType::Remote);
        assert!(message.contains("Remote Lore server is currently unavailable"));

        let message = handler.fallback_message(ProviderType::Local);
        assert!(message.contains("No fallback available"));
    }
}
