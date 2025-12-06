//! Hacker News provider

use crate::models::{FeedItem, FeedItemMetadata};
use crate::providers::{FeedProvider, ProviderError, ProviderStatus, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::future::join_all;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const HN_API_BASE: &str = "https://hacker-news.firebaseio.com/v0";

/// Hacker News story category
#[derive(Debug, Clone, Copy, Default)]
pub enum HnCategory {
    #[default]
    Top,
    New,
    Best,
    Ask,
    Show,
    Job,
}

impl HnCategory {
    pub fn endpoint(&self) -> &str {
        match self {
            HnCategory::Top => "topstories",
            HnCategory::New => "newstories",
            HnCategory::Best => "beststories",
            HnCategory::Ask => "askstories",
            HnCategory::Show => "showstories",
            HnCategory::Job => "jobstories",
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            HnCategory::Top => "Top",
            HnCategory::New => "New",
            HnCategory::Best => "Best",
            HnCategory::Ask => "Ask HN",
            HnCategory::Show => "Show HN",
            HnCategory::Job => "Jobs",
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "new" => HnCategory::New,
            "best" => HnCategory::Best,
            "ask" => HnCategory::Ask,
            "show" => HnCategory::Show,
            "job" | "jobs" => HnCategory::Job,
            _ => HnCategory::Top,
        }
    }
}

/// Hacker News item from API
#[derive(Debug, Clone, Deserialize)]
pub struct HnItem {
    pub id: u64,
    #[serde(rename = "type")]
    pub item_type: String,
    pub by: Option<String>,
    pub time: i64,
    pub text: Option<String>,
    pub dead: Option<bool>,
    pub deleted: Option<bool>,
    pub parent: Option<u64>,
    pub kids: Option<Vec<u64>>,
    pub url: Option<String>,
    pub score: Option<i32>,
    pub title: Option<String>,
    pub descendants: Option<i32>,
}

/// Hacker News provider
pub struct HackerNewsProvider {
    client: Client,
    pub category: HnCategory,
    enabled: bool,
    // Cache story IDs for infinite scroll
    cached_ids: std::sync::Mutex<Vec<u64>>,
}

impl HackerNewsProvider {
    /// Create a new Hacker News provider
    pub fn new(category: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ProviderError::Other(e.to_string()))?;
        
        Ok(Self {
            client,
            category: category.map(|c| HnCategory::from_str(&c)).unwrap_or_default(),
            enabled: true,
            cached_ids: std::sync::Mutex::new(Vec::new()),
        })
    }
    
    /// Set the current category
    pub fn set_category(&mut self, category: HnCategory) {
        self.category = category;
        // Clear cached IDs when category changes
        if let Ok(mut ids) = self.cached_ids.lock() {
            ids.clear();
        }
    }
    
    /// Fetch story IDs for current category
    async fn fetch_story_ids(&self, limit: usize) -> Result<Vec<u64>> {
        let url = format!("{}/{}.json", HN_API_BASE, self.category.endpoint());
        
        let ids: Vec<u64> = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))?;
        
        // Cache the IDs for infinite scroll
        if let Ok(mut cached) = self.cached_ids.lock() {
            *cached = ids.clone();
        }
        
        Ok(ids.into_iter().take(limit).collect())
    }
    
    /// Fetch items with offset for infinite scroll
    pub async fn fetch_items_with_offset(&self, offset: usize, limit: usize) -> Result<Vec<FeedItem>> {
        // Get cached IDs or fetch new ones
        let ids: Vec<u64> = {
            let cached = self.cached_ids.lock().ok();
            match cached {
                Some(cache) if !cache.is_empty() => cache.clone(),
                _ => self.fetch_all_story_ids().await?,
            }
        };
        
        // Get slice with offset
        let slice: Vec<u64> = ids.into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        
        if slice.is_empty() {
            return Ok(vec![]);
        }
        
        self.fetch_items_by_ids(&slice).await
    }
    
    /// Fetch all story IDs without limit (for caching)
    async fn fetch_all_story_ids(&self) -> Result<Vec<u64>> {
        let url = format!("{}/{}.json", HN_API_BASE, self.category.endpoint());
        
        let ids: Vec<u64> = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))?;
        
        // Cache the IDs
        if let Ok(mut cached) = self.cached_ids.lock() {
            *cached = ids.clone();
        }
        
        Ok(ids)
    }
    
    /// Fetch items by IDs (parallel)
    async fn fetch_items_by_ids(&self, ids: &[u64]) -> Result<Vec<FeedItem>> {
        let batch_size = 25;  // Increased from 10 for faster fetching
        let mut all_items = Vec::with_capacity(ids.len());
        
        for chunk in ids.chunks(batch_size) {
            let futures: Vec<_> = chunk.iter()
                .map(|&id| self.fetch_item(id))
                .collect();
            
            let results = join_all(futures).await;
            
            for item in results.into_iter().flatten() {
                if item.deleted.unwrap_or(false) || item.dead.unwrap_or(false) {
                    continue;
                }
                all_items.push(self.convert_to_feed_item(item));
            }
        }
        
        Ok(all_items)
    }

    /// Fetch a single item by ID
    async fn fetch_item(&self, id: u64) -> Result<HnItem> {
        let url = format!("{}/item/{}.json", HN_API_BASE, id);
        
        self.client
            .get(&url)
            .send()
            .await?
            .json::<Option<HnItem>>()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))?
            .ok_or(ProviderError::Other(format!("Item {} not found", id)))
    }
    
    /// Convert HnItem to FeedItem
    fn convert_to_feed_item(&self, item: HnItem) -> FeedItem {
        let published_at = DateTime::from_timestamp(item.time, 0)
            .unwrap_or_else(Utc::now);
        
        let source = match self.category {
            HnCategory::Ask => "Ask HN",
            HnCategory::Show => "Show HN",
            HnCategory::Job => "HN Jobs",
            _ => "Hacker News",
        };
        
        let metadata = FeedItemMetadata {
            score: item.score,
            comments: item.descendants,
            ..Default::default()
        };
        
        let mut feed_item = FeedItem::new(
            item.id.to_string(),
            self.id().to_string(),
            item.title.unwrap_or_else(|| "(no title)".to_string()),
            source.to_string(),
            published_at,
        )
        .with_metadata(metadata);
        
        if let Some(author) = item.by {
            feed_item = feed_item.with_author(author);
        }
        
        if let Some(url) = item.url {
            feed_item = feed_item.with_url(url);
        }
        
        if let Some(text) = item.text {
            // Convert HTML to plain text
            let plain_text = html2text::from_read(text.as_bytes(), 80);
            feed_item = feed_item.with_summary(plain_text);
        }
        
        feed_item
    }
}

#[async_trait]
impl FeedProvider for HackerNewsProvider {
    fn id(&self) -> &str {
        "hackernews"
    }
    
    fn name(&self) -> &str {
        "Hacker News"
    }
    
    fn description(&self) -> &str {
        "Tech news and discussions from Y Combinator"
    }
    
    fn icon(&self) -> &str {
        "🟠"
    }
    
    fn status(&self) -> ProviderStatus {
        if self.enabled {
            ProviderStatus::Ready
        } else {
            ProviderStatus::Disabled
        }
    }
    
    fn categories(&self) -> Vec<&str> {
        vec!["top", "new", "best", "ask", "show", "job"]
    }
    
    async fn fetch_items(&self, limit: usize) -> Result<Vec<FeedItem>> {
        let ids = self.fetch_story_ids(limit).await?;
        
        // Parallel fetch with batching (10 concurrent requests)
        let batch_size = 10;
        let mut all_items = Vec::with_capacity(ids.len());
        
        for chunk in ids.chunks(batch_size) {
            let futures: Vec<_> = chunk.iter()
                .map(|&id| self.fetch_item(id))
                .collect();
            
            let results = join_all(futures).await;
            
            for result in results {
                if let Ok(item) = result {
                    // Skip deleted/dead items
                    if item.deleted.unwrap_or(false) || item.dead.unwrap_or(false) {
                        continue;
                    }
                    all_items.push(self.convert_to_feed_item(item));
                }
            }
        }
        
        Ok(all_items)
    }
    
    fn supports_search(&self) -> bool {
        false  // HN API doesn't have search (would need Algolia)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_ready() {
        let provider = HackerNewsProvider::new(None).unwrap();
        assert_eq!(provider.status(), ProviderStatus::Ready);
    }
    
    #[test]
    fn test_category_parsing() {
        assert!(matches!(HnCategory::from_str("top"), HnCategory::Top));
        assert!(matches!(HnCategory::from_str("show"), HnCategory::Show));
        assert!(matches!(HnCategory::from_str("unknown"), HnCategory::Top));
    }
    
    #[tokio::test]
    async fn test_fetch_story_ids() {
        let provider = HackerNewsProvider::new(None).unwrap();
        let ids = provider.fetch_story_ids(5).await;
        assert!(ids.is_ok());
        assert!(ids.unwrap().len() <= 5);
    }
}
