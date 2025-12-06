//! Provider Registry
//! 
//! Central registry for managing feed providers

use crate::models::FeedItem;
use crate::providers::{FeedProvider, ProviderError, ProviderStatus, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Central registry for managing feed providers
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn FeedProvider>>,
    order: Vec<String>,  // Maintain insertion order
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            order: Vec::new(),
        }
    }
    
    /// Register a new provider
    pub fn register<P: FeedProvider + 'static>(&mut self, provider: P) {
        let id = provider.id().to_string();
        if !self.providers.contains_key(&id) {
            self.order.push(id.clone());
        }
        self.providers.insert(id, Arc::new(provider));
    }
    
    /// Register a provider from an Arc
    pub fn register_arc(&mut self, provider: Arc<dyn FeedProvider>) {
        let id = provider.id().to_string();
        if !self.providers.contains_key(&id) {
            self.order.push(id.clone());
        }
        self.providers.insert(id, provider);
    }
    
    /// Get a provider by ID
    pub fn get(&self, id: &str) -> Option<Arc<dyn FeedProvider>> {
        self.providers.get(id).cloned()
    }
    
    /// Get all providers in registration order
    pub fn all(&self) -> Vec<Arc<dyn FeedProvider>> {
        self.order
            .iter()
            .filter_map(|id| self.providers.get(id).cloned())
            .collect()
    }
    
    /// Get only ready/enabled providers
    pub fn ready(&self) -> Vec<Arc<dyn FeedProvider>> {
        self.all()
            .into_iter()
            .filter(|p| p.is_ready())
            .collect()
    }
    
    /// Get provider count
    pub fn len(&self) -> usize {
        self.providers.len()
    }
    
    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
    
    /// Get provider IDs
    pub fn ids(&self) -> Vec<&str> {
        self.order.iter().map(|s| s.as_str()).collect()
    }
    
    /// Remove a provider
    pub fn remove(&mut self, id: &str) -> Option<Arc<dyn FeedProvider>> {
        self.order.retain(|i| i != id);
        self.providers.remove(id)
    }
    
    /// Fetch items from all ready providers
    pub async fn fetch_all(&self, limit_per_provider: usize) -> Vec<FeedItem> {
        let providers = self.ready();
        let mut all_items = Vec::new();
        
        for provider in providers {
            match provider.fetch_items(limit_per_provider).await {
                Ok(items) => all_items.extend(items),
                Err(e) => {
                    // Log error but continue with other providers
                    tracing::warn!("Provider {} failed: {}", provider.id(), e);
                }
            }
        }
        
        // Sort by publish date (newest first)
        all_items.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        
        all_items
    }
    
    /// Fetch items from a specific provider
    pub async fn fetch_from(&self, provider_id: &str, limit: usize) -> Result<Vec<FeedItem>> {
        let provider = self.get(provider_id)
            .ok_or_else(|| ProviderError::NotConfigured(format!("Provider '{}' not found", provider_id)))?;
        
        provider.fetch_items(limit).await
    }
    
    /// Get summary of provider statuses for UI
    pub fn status_summary(&self) -> Vec<ProviderSummary> {
        self.order
            .iter()
            .filter_map(|id| {
                self.providers.get(id).map(|p| ProviderSummary {
                    id: p.id().to_string(),
                    name: p.name().to_string(),
                    icon: p.icon().to_string(),
                    description: p.description().to_string(),
                    status: p.status(),
                })
            })
            .collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of a provider for UI display
#[derive(Debug, Clone)]
pub struct ProviderSummary {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub status: ProviderStatus,
}

impl ProviderSummary {
    /// Get display line for landing page
    pub fn display_line(&self) -> String {
        format!("{} {} - {}", self.icon, self.name, self.description)
    }
    
    /// Get status indicator
    pub fn status_indicator(&self) -> &str {
        match &self.status {
            ProviderStatus::Ready => "✓",
            ProviderStatus::NeedsConfig => "⚠",
            ProviderStatus::Disabled => "○",
            ProviderStatus::Error(_) => "✗",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::ProviderStatus;
    use async_trait::async_trait;
    
    struct MockProvider {
        id: String,
        ready: bool,
    }
    
    #[async_trait]
    impl FeedProvider for MockProvider {
        fn id(&self) -> &str { &self.id }
        fn name(&self) -> &str { "Mock" }
        fn description(&self) -> &str { "Mock provider" }
        fn icon(&self) -> &str { "🧪" }
        fn status(&self) -> ProviderStatus {
            if self.ready { ProviderStatus::Ready } else { ProviderStatus::Disabled }
        }
        async fn fetch_items(&self, _limit: usize) -> Result<Vec<FeedItem>> {
            Ok(vec![])
        }
    }
    
    #[test]
    fn test_registry_register() {
        let mut registry = ProviderRegistry::new();
        registry.register(MockProvider { id: "test".to_string(), ready: true });
        assert_eq!(registry.len(), 1);
        assert!(registry.get("test").is_some());
    }
    
    #[test]
    fn test_registry_ready_filter() {
        let mut registry = ProviderRegistry::new();
        registry.register(MockProvider { id: "ready".to_string(), ready: true });
        registry.register(MockProvider { id: "disabled".to_string(), ready: false });
        
        assert_eq!(registry.all().len(), 2);
        assert_eq!(registry.ready().len(), 1);
    }
}
