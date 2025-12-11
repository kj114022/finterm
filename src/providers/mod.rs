//! Feed Provider module
//! 
//! This module provides a trait-based architecture for feed providers,
//! making it easy to add new sources (APIs, RSS, etc.) without modifying core code.

pub mod finnhub;
pub mod hackernews;
pub mod cratesio;
pub mod reddit;
pub mod registry;

use crate::models::FeedItem;
use async_trait::async_trait;
use std::fmt;
use thiserror::Error;

/// Errors that can occur when fetching from a provider
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Authentication failed: {0}")]
    Auth(String),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Provider not configured: {0}")]
    NotConfigured(String),
    
    #[error("Provider error: {0}")]
    Other(String),
}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        ProviderError::Network(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ProviderError>;

/// Provider status for UI display
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderStatus {
    /// Provider is configured and ready
    Ready,
    /// Provider needs configuration (e.g., API key)
    NeedsConfig,
    /// Provider is disabled in settings
    Disabled,
    /// Provider encountered an error
    Error(String),
}

impl fmt::Display for ProviderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderStatus::Ready => write!(f, "✓ Ready"),
            ProviderStatus::NeedsConfig => write!(f, "⚠ Needs Config"),
            ProviderStatus::Disabled => write!(f, "○ Disabled"),
            ProviderStatus::Error(e) => write!(f, "✗ Error: {}", e),
        }
    }
}

/// Common trait for all feed providers
#[async_trait]
pub trait FeedProvider: Send + Sync {
    /// Unique identifier for this provider (e.g., "finnhub", "hackernews")
    fn id(&self) -> &str;
    
    /// Display name for UI (e.g., "Finnhub", "Hacker News")
    fn name(&self) -> &str;
    
    /// Short description of the provider
    fn description(&self) -> &str;
    
    /// Icon/emoji for the provider
    fn icon(&self) -> &str;
    
    /// Current status of the provider
    fn status(&self) -> ProviderStatus;
    
    /// Check if provider is ready to fetch
    fn is_ready(&self) -> bool {
        self.status() == ProviderStatus::Ready
    }
    
    /// Fetch latest items from the feed
    async fn fetch_items(&self, limit: usize) -> Result<Vec<FeedItem>>;
    
    /// Fetch items with offset for infinite scroll (default: not supported)
    async fn fetch_items_with_offset(&self, _offset: usize, _limit: usize) -> Result<Vec<FeedItem>> {
        Err(ProviderError::Other("Offset not supported".to_string()))
    }
    
    /// Check if provider supports infinite scroll
    fn supports_offset(&self) -> bool {
        false
    }
    
    /// Search items (optional capability - default returns empty)
    async fn search(&self, _query: &str, _limit: usize) -> Result<Vec<FeedItem>> {
        Ok(vec![])
    }
    
    /// Check if provider supports search
    fn supports_search(&self) -> bool {
        false
    }
    
    /// Get available categories/feeds for this provider (if any)
    fn categories(&self) -> Vec<&str> {
        vec![]
    }
}

// Re-export main types
pub use registry::ProviderRegistry;
pub use finnhub::FinnhubProvider;
pub use hackernews::HackerNewsProvider;
pub use cratesio::CratesIoProvider;
pub use reddit::RedditProvider;
