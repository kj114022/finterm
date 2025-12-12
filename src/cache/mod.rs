use crate::models::{CacheEntry, CacheKey, CacheStats};
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Database error: {0}")]
    Database(#[from] sled::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Cache entry expired")]
    Expired,

    #[error("Cache entry not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, CacheError>;

/// Sled-based cache manager
pub struct CacheManager {
    db: sled::Db,
    stats: CacheStats,
    max_size_bytes: u64,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(cache_path: PathBuf, max_size_mb: u64) -> Result<Self> {
        // Try to open database, if it fails due to lock or corruption, clear and retry
        let db = match sled::open(&cache_path) {
            Ok(db) => db,
            Err(e) => {
                tracing::warn!("Cache database error, clearing and retrying: {}", e);
                // Remove corrupted/locked database
                let _ = std::fs::remove_dir_all(&cache_path);
                // Retry opening
                sled::open(&cache_path)?
            }
        };

        Ok(Self {
            db,
            stats: CacheStats::default(),
            max_size_bytes: max_size_mb * 1024 * 1024,
        })
    }

    /// Get a cached value
    pub fn get<T: DeserializeOwned>(&mut self, key: CacheKey) -> Result<T> {
        let key_str = key.as_cache_key();

        match self.db.get(&key_str)? {
            Some(bytes) => {
                let entry: CacheEntry<T> = serde_json::from_slice(&bytes)
                    .map_err(|e| CacheError::Serialization(e.to_string()))?;

                if entry.is_valid() {
                    self.stats.hits += 1;
                    Ok(entry.data)
                } else {
                    // Remove expired entry
                    self.db.remove(&key_str)?;
                    self.stats.misses += 1;
                    Err(CacheError::Expired)
                }
            }
            None => {
                self.stats.misses += 1;
                Err(CacheError::NotFound)
            }
        }
    }

    /// Set a cached value with TTL
    pub fn set<T: Serialize>(&mut self, key: CacheKey, value: T, ttl_seconds: u64) -> Result<()> {
        let entry = CacheEntry::new(value, ttl_seconds);
        let bytes =
            serde_json::to_vec(&entry).map_err(|e| CacheError::Serialization(e.to_string()))?;

        let key_str = key.as_cache_key();
        self.db.insert(&key_str, bytes)?;

        // Check cache size and evict if necessary
        self.evict_if_needed()?;

        Ok(())
    }

    /// Remove a cache entry
    pub fn remove(&mut self, key: CacheKey) -> Result<()> {
        let key_str = key.as_cache_key();
        self.db.remove(&key_str)?;
        Ok(())
    }

    /// Clear all cache entries
    pub fn clear(&mut self) -> Result<()> {
        self.db.clear()?;
        self.stats = CacheStats::default();
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&mut self) -> CacheStats {
        self.stats.total_entries = self.db.len();
        self.stats.total_size_bytes = self.calculate_size();
        self.stats.clone()
    }

    /// Calculate total cache size
    fn calculate_size(&self) -> u64 {
        self.db
            .iter()
            .filter_map(|r| r.ok())
            .map(|(k, v)| (k.len() + v.len()) as u64)
            .sum()
    }

    /// Evict entries if cache is over size limit (LRU)
    fn evict_if_needed(&mut self) -> Result<()> {
        let current_size = self.calculate_size();

        if current_size > self.max_size_bytes {
            // Collect all entries with their cached_at times
            let mut entries: Vec<(Vec<u8>, i64)> = Vec::new();

            for item in self.db.iter().filter_map(|r| r.ok()) {
                let (key, value) = item;

                // Try to parse the entry to get cached_at time
                if let Ok(entry) = serde_json::from_slice::<CacheEntry<serde_json::Value>>(&value) {
                    entries.push((key.to_vec(), entry.cached_at.timestamp()));
                }
            }

            // Sort by timestamp (oldest first)
            entries.sort_by_key(|(_, ts)| *ts);

            // Remove oldest 25% of entries
            let evict_count = (entries.len() / 4).max(1);
            for (key, _) in entries.iter().take(evict_count) {
                self.db.remove(key)?;
                self.stats.evictions += 1;
            }
        }

        Ok(())
    }

    /// Flush to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

impl Drop for CacheManager {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_basic_operations() {
        let dir = tempdir().unwrap();
        let mut cache = CacheManager::new(dir.path().to_path_buf(), 10).unwrap();

        let key = CacheKey::HnStory(12345);
        cache
            .set(key.clone(), "test data".to_string(), 3600)
            .unwrap();

        let result: String = cache.get(key).unwrap();
        assert_eq!(result, "test data");
    }
}
