use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub cached_at: DateTime<Utc>,
    pub ttl_seconds: u64,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry
    pub fn new(data: T, ttl_seconds: u64) -> Self {
        Self {
            data,
            cached_at: Utc::now(),
            ttl_seconds,
        }
    }
    
    /// Check if the cache entry is still valid
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.cached_at);
        age.num_seconds() < self.ttl_seconds as i64
    }
    
    /// Get remaining TTL in seconds
    pub fn remaining_ttl(&self) -> i64 {
        let now = Utc::now();
        let age = now.signed_duration_since(self.cached_at);
        self.ttl_seconds as i64 - age.num_seconds()
    }
}

/// Cache key types for organizing cached data
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
    AletheiaArticle(String),
    AletheiaSearch(String),
    HnStory(u64),
    HnComments(u64),
    HnContent(String), // URL hash
    HnStoryList(String), // Category name
}

impl CacheKey {
    /// Convert cache key to string for storage
    pub fn to_string(&self) -> String {
        match self {
            CacheKey::AletheiaArticle(id) => format!("aletheia:article:{}", id),
            CacheKey::AletheiaSearch(query) => {
                let hash = Self::hash_string(query);
                format!("aletheia:search:{}", hash)
            }
            CacheKey::HnStory(id) => format!("hn:story:{}", id),
            CacheKey::HnComments(id) => format!("hn:comments:{}", id),
            CacheKey::HnContent(url) => {
                let hash = Self::hash_string(url);
                format!("hn:content:{}", hash)
            }
            CacheKey::HnStoryList(category) => format!("hn:list:{}", category),
        }
    }
    
    /// Hash a string to create a stable key
    fn hash_string(s: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..8]) // Use first 8 bytes
    }
}

/// Statistics about cache usage
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

impl CacheStats {
    /// Calculate hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            (self.hits as f64 / (self.hits + self.misses) as f64) * 100.0
        }
    }
}
