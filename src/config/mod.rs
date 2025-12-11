use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
    
    #[error("Config validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct Config {
    #[serde(default)]
    pub finnhub: FinnhubConfig,
    
    #[serde(default)]
    pub hackernews: HackerNewsConfig,
    
    #[serde(default)]
    pub reddit: RedditConfig,
    
    #[serde(default)]
    pub ui: UiConfig,
    
    #[serde(default)]
    pub cache: CacheConfig,
    
    #[serde(default)]
    pub keybindings: KeybindingsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinnhubConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_finnhub_url")]
    pub base_url: String,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u64,
    #[serde(default = "default_max_articles")]
    pub max_articles: usize,
    #[serde(default = "default_news_category")]
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HackerNewsConfig {
    #[serde(default = "default_max_stories")]
    pub max_stories: usize,
    #[serde(default = "default_categories")]
    pub categories: Vec<String>,
    #[serde(default)]
    pub include_dead: bool,
    #[serde(default = "default_true")]
    pub fetch_full_content: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditConfig {
    #[serde(default = "default_subreddits")]
    pub subreddits: Vec<String>,
    #[serde(default = "default_reddit_sort")]
    pub sort: String,
    #[serde(default = "default_max_posts")]
    pub max_posts: usize,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_true")]
    pub vim_mode: bool,
    #[serde(default = "default_true")]
    pub show_help: bool,
    #[serde(default = "default_view")]
    pub default_view: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_cache_ttl")]
    pub ttl: u64,
    #[serde(default = "default_max_cache_size")]
    pub max_size_mb: u64,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    #[serde(default = "default_quit_key")]
    pub quit: String,
    #[serde(default = "default_search_key")]
    pub search: String,
    #[serde(default = "default_help_key")]
    pub help: String,
}

// Default value functions
fn default_finnhub_url() -> String {
    "https://finnhub.io/api/v1".to_string()
}

fn default_refresh_interval() -> u64 {
    300
}

fn default_max_articles() -> usize {
    50
}

fn default_news_category() -> String {
    "general".to_string()
}

fn default_max_stories() -> usize {
    50
}

fn default_categories() -> Vec<String> {
    vec!["top".to_string(), "new".to_string(), "show".to_string(), "ask".to_string()]
}

fn default_subreddits() -> Vec<String> {
    vec!["technology".to_string(), "programming".to_string(), "rust".to_string(), "finance".to_string()]
}

fn default_reddit_sort() -> String {
    "hot".to_string()
}

fn default_max_posts() -> usize {
    50
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_view() -> String {
    "dashboard".to_string()
}

fn default_cache_ttl() -> u64 {
    3600
}

fn default_max_cache_size() -> u64 {
    100
}

fn default_quit_key() -> String {
    "q".to_string()
}

fn default_search_key() -> String {
    "/".to_string()
}

fn default_help_key() -> String {
    "?".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for FinnhubConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: default_finnhub_url(),
            refresh_interval: default_refresh_interval(),
            max_articles: default_max_articles(),
            category: default_news_category(),
        }
    }
}

impl Default for HackerNewsConfig {
    fn default() -> Self {
        Self {
            max_stories: default_max_stories(),
            categories: default_categories(),
            include_dead: false,
            fetch_full_content: true,
        }
    }
}

impl Default for RedditConfig {
    fn default() -> Self {
        Self {
            subreddits: default_subreddits(),
            sort: default_reddit_sort(),
            max_posts: default_max_posts(),
            enabled: true,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            vim_mode: true,
            show_help: true,
            default_view: default_view(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl: default_cache_ttl(),
            max_size_mb: default_max_cache_size(),
            path: None,
        }
    }
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            quit: default_quit_key(),
            search: default_search_key(),
            help: default_help_key(),
        }
    }
}


impl Config {
    /// Load configuration from file
    pub fn load(path: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::Validation(e.to_string()))?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, contents)?;
        Ok(())
    }
    
    /// Validate configuration
    fn validate(&self) -> Result<()> {
        // API key is optional - if not set, Finnhub will be skipped (demo mode)
        
        if self.cache.max_size_mb == 0 {
            return Err(ConfigError::Validation(
                "Cache max size must be greater than 0".to_string(),
            ));
        }
        
        Ok(())
    }
    
    /// Get default config file path
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("finterm")
            .join("config.toml")
    }
    
    /// Get cache directory
    pub fn cache_dir(&self) -> PathBuf {
        if let Some(path) = &self.cache.path {
            PathBuf::from(shellexpand::tilde(path).to_string())
        } else {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("finterm")
        }
    }
    
    /// Create example configuration file
    pub fn create_example() -> String {
        toml::to_string_pretty(&Config::default()).unwrap()
    }
}
