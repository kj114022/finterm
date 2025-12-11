//! Crates.io provider
//! 
//! Displays the latest Rust crates from crates.io

use crate::models::{FeedItem, FeedItemMetadata};
use crate::providers::{FeedProvider, ProviderError, ProviderStatus, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const CRATES_IO_API: &str = "https://crates.io/api/v1";
const USER_AGENT: &str = "finterm/0.1.0 (https://github.com/finterm)";

/// Crates.io feed category
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum CratesCategory {
    #[default]
    New,
    JustUpdated,
    MostDownloaded,
    RecentlyDownloaded,
}

impl CratesCategory {
    pub fn as_str(&self) -> &str {
        match self {
            CratesCategory::New => "New",
            CratesCategory::JustUpdated => "Just Updated",
            CratesCategory::MostDownloaded => "Most Downloaded",
            CratesCategory::RecentlyDownloaded => "Recently Downloaded",
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "updated" | "just_updated" | "justupdated" => CratesCategory::JustUpdated,
            "downloaded" | "most_downloaded" | "mostdownloaded" => CratesCategory::MostDownloaded,
            "recent" | "recently_downloaded" | "recentlydownloaded" => CratesCategory::RecentlyDownloaded,
            _ => CratesCategory::New,
        }
    }
}

/// Crate item from crates.io API
#[derive(Debug, Clone, Deserialize)]
pub struct CrateItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub downloads: i64,
    pub recent_downloads: Option<i64>,
    pub max_version: String,
    pub newest_version: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
}



/// Crates.io provider
pub struct CratesIoProvider {
    client: Client,
    pub category: CratesCategory,
    enabled: bool,
}

impl CratesIoProvider {
    /// Create a new crates.io provider
    pub fn new(category: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| ProviderError::Other(e.to_string()))?;
        
        Ok(Self {
            client,
            category: category.map(|c| CratesCategory::from_str(&c)).unwrap_or_default(),
            enabled: true,
        })
    }
    
    /// Set the current category
    pub fn set_category(&mut self, category: CratesCategory) {
        self.category = category;
    }

    
    /// Convert CrateItem to FeedItem
    fn convert_to_feed_item(&self, crate_item: CrateItem) -> FeedItem {
        let source = match self.category {
            CratesCategory::New => "New Crates",
            CratesCategory::JustUpdated => "Updated Crates",
            CratesCategory::MostDownloaded => "Popular Crates",
            CratesCategory::RecentlyDownloaded => "Trending Crates",
        };
        
        let metadata = FeedItemMetadata {
            score: Some(crate_item.downloads as i32),
            tags: vec!["rust".to_string(), "crate".to_string()],
            ..Default::default()
        };
        
        let url = format!("https://crates.io/crates/{}", crate_item.name);
        
        let mut feed_item = FeedItem::new(
            crate_item.id.clone(),
            self.id().to_string(),
            format!("{} v{}", crate_item.name, crate_item.newest_version),
            source.to_string(),
            crate_item.updated_at,
        )
        .with_metadata(metadata)
        .with_url(url);
        
        if let Some(desc) = crate_item.description {
            feed_item = feed_item.with_summary(desc);
        }
        
        feed_item
    }
}

#[async_trait]
impl FeedProvider for CratesIoProvider {
    fn id(&self) -> &str {
        "cratesio"
    }
    
    fn name(&self) -> &str {
        "Crates.io"
    }
    
    fn description(&self) -> &str {
        "The Rust community's crate registry"
    }
    
    fn icon(&self) -> &str {
        "[CR]"
    }
    
    fn status(&self) -> ProviderStatus {
        if self.enabled {
            ProviderStatus::Ready
        } else {
            ProviderStatus::Disabled
        }
    }
    
    fn categories(&self) -> Vec<&str> {
        vec!["new", "updated", "downloaded", "recent"]
    }
    
    async fn fetch_items(&self, limit: usize) -> Result<Vec<FeedItem>> {
        // Use paginated API for larger feeds (summary only returns 10)
        let sort = match self.category {
            CratesCategory::New => "new",
            CratesCategory::JustUpdated => "recent-updates",
            CratesCategory::MostDownloaded => "downloads",
            CratesCategory::RecentlyDownloaded => "recent-downloads",
        };
        
        let per_page = limit.min(100); // crates.io max is 100
        let url = format!("{}/crates?sort={}&per_page={}", CRATES_IO_API, sort, per_page);
        
        #[derive(Deserialize)]
        struct CratesResponse {
            crates: Vec<CrateItem>,
        }
        
        let response: CratesResponse = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))?;
        
        let items: Vec<FeedItem> = response.crates
            .into_iter()
            .take(limit)
            .map(|c| self.convert_to_feed_item(c))
            .collect();
        
        Ok(items)
    }
    
    fn supports_search(&self) -> bool {
        true
    }
    
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<FeedItem>> {
        let url = format!("{}/crates?q={}&per_page={}", CRATES_IO_API, query, limit);
        
        #[derive(Deserialize)]
        struct SearchResponse {
            crates: Vec<CrateItem>,
        }
        
        let response: SearchResponse = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))?;
        
        let items: Vec<FeedItem> = response.crates
            .into_iter()
            .take(limit)
            .map(|c| self.convert_to_feed_item(c))
            .collect();
        
        Ok(items)
    }
    
    fn supports_offset(&self) -> bool {
        true  // Crates.io supports pagination
    }
    
    async fn fetch_items_with_offset(&self, offset: usize, limit: usize) -> Result<Vec<FeedItem>> {
        let sort = match self.category {
            CratesCategory::New => "new",
            CratesCategory::JustUpdated => "recent-updates",
            CratesCategory::MostDownloaded => "downloads",
            CratesCategory::RecentlyDownloaded => "recent-downloads",
        };
        
        // Calculate page from offset (crates.io uses 1-based pages)
        let per_page = limit.min(100);
        let page = (offset / per_page) + 1;
        
        let url = format!(
            "{}/crates?sort={}&per_page={}&page={}",
            CRATES_IO_API, sort, per_page, page
        );
        
        #[derive(Deserialize)]
        struct CratesResponse {
            crates: Vec<CrateItem>,
        }
        
        let response: CratesResponse = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))?;
        
        let items: Vec<FeedItem> = response.crates
            .into_iter()
            .take(limit)
            .map(|c| self.convert_to_feed_item(c))
            .collect();
        
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_ready() {
        let provider = CratesIoProvider::new(None).unwrap();
        assert_eq!(provider.status(), ProviderStatus::Ready);
    }
    
    #[test]
    fn test_category_parsing() {
        assert!(matches!(CratesCategory::from_str("new"), CratesCategory::New));
        assert!(matches!(CratesCategory::from_str("updated"), CratesCategory::JustUpdated));
        assert!(matches!(CratesCategory::from_str("downloaded"), CratesCategory::MostDownloaded));
    }
    
    #[tokio::test]
    async fn test_fetch_items() {
        let provider = CratesIoProvider::new(None).unwrap();
        let result = provider.fetch_items(10).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }
}
