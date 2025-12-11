//! Finnhub financial news provider

use crate::models::feed_item::{FeedItem, FeedItemMetadata};
use crate::providers::{FeedProvider, ProviderError, ProviderStatus, Result};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

/// Raw news item from Finnhub API
#[derive(Debug, Deserialize)]
struct FinnhubNewsItem {
    category: String,
    datetime: i64,
    headline: String,
    id: u64,
    image: String,
    related: String,
    source: String,
    summary: String,
    url: String,
}

/// News category for Finnhub API
#[derive(Debug, Clone, Copy, Default)]
pub enum NewsCategory {
    #[default]
    General,
    Forex,
    Crypto,
    Merger,
}

impl NewsCategory {
    pub fn as_str(&self) -> &str {
        match self {
            NewsCategory::General => "general",
            NewsCategory::Forex => "forex",
            NewsCategory::Crypto => "crypto",
            NewsCategory::Merger => "merger",
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "forex" => NewsCategory::Forex,
            "crypto" => NewsCategory::Crypto,
            "merger" => NewsCategory::Merger,
            _ => NewsCategory::General,
        }
    }
}

/// Finnhub financial news provider
pub struct FinnhubProvider {
    client: Client,
    api_key: String,
    base_url: String,
    category: NewsCategory,
}

impl FinnhubProvider {
    /// Create a new Finnhub provider
    pub fn new(api_key: String, category: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ProviderError::Other(e.to_string()))?;
        
        Ok(Self {
            client,
            api_key,
            base_url: "https://finnhub.io/api/v1".to_string(),
            category: category.map(|c| NewsCategory::from_str(&c)).unwrap_or_default(),
        })
    }
    
    /// Convert Finnhub news item to FeedItem (raw, no filtering)
    fn convert_to_feed_item(&self, item: FinnhubNewsItem) -> FeedItem {
        let published_at = Utc.timestamp_opt(item.datetime, 0)
            .single()
            .unwrap_or_else(Utc::now);
        
        // Extract related symbols as tags (raw, no filtering)
        let tags: Vec<String> = if item.related.is_empty() {
            vec![item.category.clone()]
        } else {
            item.related.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        
        let metadata = FeedItemMetadata {
            tags,
            image_url: if item.image.is_empty() { None } else { Some(item.image) },
            ..Default::default()
        };
        
        FeedItem::new(
            item.id.to_string(),
            self.id().to_string(),
            item.headline,
            item.source,
            published_at,
        )
        .with_summary(item.summary)
        .with_url(item.url)
        .with_metadata(metadata)
    }
}

#[async_trait]
impl FeedProvider for FinnhubProvider {
    fn id(&self) -> &str {
        "finnhub"
    }
    
    fn name(&self) -> &str {
        "Finnhub"
    }
    
    fn description(&self) -> &str {
        "Real-time financial news from markets worldwide"
    }
    
    fn icon(&self) -> &str {
        "[FH]"
    }
    
    fn status(&self) -> ProviderStatus {
        if self.api_key.is_empty() {
            ProviderStatus::NeedsConfig
        } else {
            ProviderStatus::Ready
        }
    }
    
    fn categories(&self) -> Vec<&str> {
        vec!["general", "forex", "crypto", "merger"]
    }
    
    async fn fetch_items(&self, limit: usize) -> Result<Vec<FeedItem>> {
        if self.api_key.is_empty() {
            return Err(ProviderError::NotConfigured("Finnhub API key not set".to_string()));
        }
        
        let url = format!(
            "{}/news?category={}&token={}",
            self.base_url,
            self.category.as_str(),
            self.api_key
        );
        
        let response = self.client.get(&url).send().await?;
        
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::Auth("Invalid API key".to_string()));
        }
        
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimit);
        }
        
        let items: Vec<FinnhubNewsItem> = response.json().await
            .map_err(|e| ProviderError::Parse(e.to_string()))?;
        
        let feed_items: Vec<FeedItem> = items
            .into_iter()
            .take(limit)
            .map(|item| self.convert_to_feed_item(item))
            .collect();
        
        Ok(feed_items)
    }
    
    fn supports_search(&self) -> bool {
        false  // Finnhub free tier doesn't have search
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_status_no_key() {
        let provider = FinnhubProvider::new("".to_string(), None).unwrap();
        assert_eq!(provider.status(), ProviderStatus::NeedsConfig);
    }
    
    #[test]
    fn test_provider_status_with_key() {
        let provider = FinnhubProvider::new("test_key".to_string(), None).unwrap();
        assert_eq!(provider.status(), ProviderStatus::Ready);
    }
    
    #[test]
    fn test_category_parsing() {
        assert!(matches!(NewsCategory::from_str("general"), NewsCategory::General));
        assert!(matches!(NewsCategory::from_str("crypto"), NewsCategory::Crypto));
        assert!(matches!(NewsCategory::from_str("unknown"), NewsCategory::General));
    }
}
