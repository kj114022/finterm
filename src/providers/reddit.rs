//! Reddit RSS provider
//!
//! Fetches posts from Reddit subreddits via their public RSS feeds.

use crate::models::{FeedItem, FeedItemMetadata, Comment};
use crate::providers::{FeedProvider, ProviderError, ProviderStatus, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::future::join_all;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::Client;
use std::time::Duration;

const REDDIT_BASE_URL: &str = "https://www.reddit.com";

/// Reddit feed sort type
#[derive(Debug, Clone, Copy, Default)]
pub enum RedditSort {
    #[default]
    Hot,
    New,
    Top,
    Rising,
}

impl RedditSort {
    pub fn as_path(&self) -> &str {
        match self {
            RedditSort::Hot => "",
            RedditSort::New => "/new",
            RedditSort::Top => "/top",
            RedditSort::Rising => "/rising",
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            RedditSort::Hot => "Hot",
            RedditSort::New => "New",
            RedditSort::Top => "Top",
            RedditSort::Rising => "Rising",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "new" => RedditSort::New,
            "top" => RedditSort::Top,
            "rising" => RedditSort::Rising,
            _ => RedditSort::Hot,
        }
    }
}

/// Reddit RSS entry parsed from feed
#[derive(Debug, Clone, Default)]
struct RedditEntry {
    id: String,
    title: String,
    author: Option<String>,
    link: Option<String>,
    published: Option<String>,
    content: Option<String>,
}

/// Reddit RSS provider
pub struct RedditProvider {
    client: Client,
    subreddits: Vec<String>,
    sort: RedditSort,
    enabled: bool,
    current_subreddit_index: usize,
}

impl RedditProvider {
    /// Create a new Reddit provider
    pub fn new(subreddits: Vec<String>, sort: Option<String>, enabled: bool) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))  // Reduced for faster response
            .connect_timeout(Duration::from_secs(5))
            .user_agent("finterm/0.1.0")
            .build()
            .map_err(|e| ProviderError::Other(e.to_string()))?;

        let default_subreddits = if subreddits.is_empty() {
            vec![
                "technology".to_string(),
                "programming".to_string(),
                "rust".to_string(),
            ]
        } else {
            subreddits
        };

        Ok(Self {
            client,
            subreddits: default_subreddits,
            sort: sort.map(|s| RedditSort::from_str(&s)).unwrap_or_default(),
            enabled,
            current_subreddit_index: 0,
        })
    }

    /// Get current subreddit
    pub fn current_subreddit(&self) -> &str {
        self.subreddits
            .get(self.current_subreddit_index)
            .map(|s| s.as_str())
            .unwrap_or("technology")
    }

    /// Set current subreddit by index
    pub fn set_subreddit_index(&mut self, index: usize) {
        if index < self.subreddits.len() {
            self.current_subreddit_index = index;
        }
    }

    /// Build RSS feed URL for a subreddit
    fn build_feed_url(&self, subreddit: &str) -> String {
        format!(
            "{}/r/{}{}.rss",
            REDDIT_BASE_URL,
            subreddit,
            self.sort.as_path()
        )
    }

    /// Fetch RSS feed from a subreddit
    async fn fetch_feed(&self, subreddit: &str) -> Result<String> {
        let url = self.build_feed_url(subreddit);

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .text()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        Ok(response)
    }

    /// Parse RSS feed XML into entries
    fn parse_feed(&self, xml: &str, subreddit: &str) -> Result<Vec<FeedItem>> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let mut items = Vec::new();
        let mut current_entry: Option<RedditEntry> = None;
        let mut current_tag = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    current_tag = tag_name.clone();

                    if tag_name == "entry" {
                        current_entry = Some(RedditEntry::default());
                    } else if tag_name == "link" && current_entry.is_some() {
                        // Extract href attribute from link element
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"href" {
                                if let Ok(href) = String::from_utf8(attr.value.to_vec()) {
                                    if let Some(ref mut entry) = current_entry {
                                        entry.link = Some(href);
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if tag_name == "entry" {
                        if let Some(entry) = current_entry.take() {
                            if let Some(item) = self.convert_entry_to_feed_item(entry, subreddit) {
                                items.push(item);
                            }
                        }
                    }
                    current_tag.clear();
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref mut entry) = current_entry {
                        let text = e.unescape().unwrap_or_default().to_string();

                        match current_tag.as_str() {
                            "title" => entry.title = text,
                            "id" => entry.id = text,
                            "published" | "updated" => {
                                if entry.published.is_none() {
                                    entry.published = Some(text);
                                }
                            }
                            "name" => entry.author = Some(text),
                            "content" => entry.content = Some(text),
                            _ => {}
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ProviderError::Parse(format!(
                        "XML parse error: {}",
                        e
                    )));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(items)
    }

    /// Convert a parsed RSS entry to a FeedItem
    fn convert_entry_to_feed_item(&self, entry: RedditEntry, subreddit: &str) -> Option<FeedItem> {
        if entry.title.is_empty() {
            return None;
        }

        let published_at = entry
            .published
            .as_ref()
            .and_then(|p| DateTime::parse_from_rfc3339(p).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        // Try to extract score and comments from content HTML
        let (score, comments) = entry
            .content
            .as_ref()
            .map(|c| self.extract_metadata_from_content(c))
            .unwrap_or((None, None));

        let metadata = FeedItemMetadata {
            score,
            comments,
            tags: vec![format!("r/{}", subreddit)],
            subreddit: Some(subreddit.to_string()),
            reddit_id: Some(self.extract_post_id(&entry.id)),
            ..Default::default()
        };

        let source = format!("r/{}", subreddit);

        let mut item = FeedItem::new(
            entry.id.clone(),
            self.id().to_string(),
            entry.title,
            source,
            published_at,
        )
        .with_metadata(metadata);

        if let Some(author) = entry.author {
            // Reddit author names come as /u/username, clean it up
            let clean_author = author.trim_start_matches("/u/").to_string();
            item = item.with_author(clean_author);
        }

        if let Some(url) = entry.link {
            item = item.with_url(url);
        }

        if let Some(content) = entry.content {
            // Convert HTML content to plain text for summary
            let plain_text = html2text::from_read(content.as_bytes(), 80);
            let summary = plain_text.lines().take(3).collect::<Vec<_>>().join(" ");
            if !summary.trim().is_empty() {
                item = item.with_summary(summary);
            }
        }

        Some(item)
    }

    /// Extract score and comments count from Reddit content HTML
    fn extract_metadata_from_content(&self, content: &str) -> (Option<i32>, Option<i32>) {
        // Reddit RSS content often contains score and comments info
        // This is a best-effort extraction
        let score = self.extract_number_after(content, "points");
        let comments = self.extract_number_after(content, "comment");

        (score, comments)
    }

    /// Helper to extract a number appearing before a keyword
    fn extract_number_after(&self, text: &str, keyword: &str) -> Option<i32> {
        // Look for patterns like "123 points" or "45 comments"
        let lower = text.to_lowercase();
        if let Some(idx) = lower.find(keyword) {
            // Look backwards from the keyword to find a number
            let before = &text[..idx];
            let words: Vec<&str> = before.split_whitespace().collect();
            if let Some(last_word) = words.last() {
                // Clean the word and try to parse as number
                let cleaned: String = last_word.chars().filter(|c| c.is_ascii_digit()).collect();
                return cleaned.parse().ok();
            }
        }
        None
    }

    /// Fetch items from all configured subreddits in parallel
    async fn fetch_from_all_subreddits(&self, limit: usize) -> Result<Vec<FeedItem>> {
        let items_per_sub = (limit / self.subreddits.len()).max(10);
        
        // Fetch all subreddits in parallel for real-time performance
        let futures: Vec<_> = self.subreddits
            .iter()
            .map(|subreddit| self.fetch_single_subreddit(subreddit.clone(), items_per_sub))
            .collect();
        
        let results = join_all(futures).await;
        
        let mut all_items: Vec<FeedItem> = results
            .into_iter()
            .flatten()
            .collect();

        // Sort by publish date (newest first)
        all_items.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        all_items.truncate(limit);

        Ok(all_items)
    }
    
    /// Fetch items from a single subreddit
    async fn fetch_single_subreddit(&self, subreddit: String, limit: usize) -> Vec<FeedItem> {
        match self.fetch_feed(&subreddit).await {
            Ok(xml) => {
                match self.parse_feed(&xml, &subreddit) {
                    Ok(mut items) => {
                        items.truncate(limit);
                        items
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse r/{}: {}", subreddit, e);
                        vec![]
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch r/{}: {}", subreddit, e);
                vec![]
            }
        }
    }
    
    /// Extract post ID from Reddit entry ID URL
    fn extract_post_id(&self, entry_id: &str) -> String {
        // Entry ID is like: https://www.reddit.com/r/rust/comments/abc123/...
        entry_id
            .split("/comments/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap_or(entry_id)
            .to_string()
    }
    
    /// Fetch comments for a Reddit post using JSON API
    pub async fn fetch_comments(&self, subreddit: &str, post_id: &str, max_depth: u32) -> Result<Vec<Comment>> {
        let url = format!("{}/r/{}/comments/{}.json", REDDIT_BASE_URL, subreddit, post_id);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .text()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;
        
        // Parse JSON response
        let json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| ProviderError::Parse(e.to_string()))?;
        
        // Reddit returns an array: [post_data, comments_data]
        let comments_listing = json
            .get(1)
            .and_then(|v| v.get("data"))
            .and_then(|v| v.get("children"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        
        let comments: Vec<Comment> = comments_listing
            .into_iter()
            .filter_map(|child| self.parse_reddit_comment(&child, 0, max_depth))
            .collect();
        
        Ok(comments)
    }
    
    /// Parse a Reddit comment from JSON
    fn parse_reddit_comment(&self, json: &serde_json::Value, depth: u32, max_depth: u32) -> Option<Comment> {
        if depth > max_depth {
            return None;
        }
        
        let kind = json.get("kind")?.as_str()?;
        if kind != "t1" {  // t1 = comment
            return None;
        }
        
        let data = json.get("data")?;
        
        let id = data.get("id")?.as_str()?.to_string();
        let author = data.get("author")?.as_str()?.to_string();
        let body = data.get("body")?.as_str()?.to_string();
        let score = data.get("score").and_then(|v| v.as_i64()).map(|v| v as i32);
        let created_utc = data.get("created_utc").and_then(|v| v.as_f64()).unwrap_or(0.0);
        
        let created_at = DateTime::from_timestamp(created_utc as i64, 0)
            .unwrap_or_else(Utc::now);
        
        let body_plain = html2text::from_read(body.as_bytes(), 80);
        
        let mut comment = Comment::new(id, author, body.clone(), created_at);
        comment.text_plain = Some(body_plain);
        comment.score = score;
        comment.depth = depth;
        
        // Parse replies
        if depth < max_depth {
            if let Some(replies) = data.get("replies").and_then(|v| v.get("data")).and_then(|v| v.get("children")).and_then(|v| v.as_array()) {
                comment.replies = replies
                    .iter()
                    .filter_map(|r| self.parse_reddit_comment(r, depth + 1, max_depth))
                    .collect();
            }
        }
        
        Some(comment)
    }
}

#[async_trait]
impl FeedProvider for RedditProvider {
    fn id(&self) -> &str {
        "reddit"
    }

    fn name(&self) -> &str {
        "Reddit"
    }

    fn description(&self) -> &str {
        "Posts from Reddit subreddits"
    }

    fn icon(&self) -> &str {
        "R"
    }

    fn status(&self) -> ProviderStatus {
        if self.enabled {
            ProviderStatus::Ready
        } else {
            ProviderStatus::Disabled
        }
    }

    fn categories(&self) -> Vec<&str> {
        self.subreddits.iter().map(|s| s.as_str()).collect()
    }

    async fn fetch_items(&self, limit: usize) -> Result<Vec<FeedItem>> {
        self.fetch_from_all_subreddits(limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_id() {
        let provider =
            RedditProvider::new(vec!["rust".to_string()], None, true).unwrap();
        assert_eq!(provider.id(), "reddit");
    }

    #[test]
    fn test_sort_from_str() {
        assert!(matches!(RedditSort::from_str("hot"), RedditSort::Hot));
        assert!(matches!(RedditSort::from_str("new"), RedditSort::New));
        assert!(matches!(RedditSort::from_str("top"), RedditSort::Top));
        assert!(matches!(RedditSort::from_str("rising"), RedditSort::Rising));
        assert!(matches!(RedditSort::from_str("unknown"), RedditSort::Hot));
    }

    #[test]
    fn test_build_feed_url() {
        let provider =
            RedditProvider::new(vec!["rust".to_string()], Some("new".to_string()), true)
                .unwrap();
        let url = provider.build_feed_url("rust");
        assert_eq!(url, "https://www.reddit.com/r/rust/new.rss");
    }

    #[test]
    fn test_default_subreddits() {
        let provider = RedditProvider::new(vec![], None, true).unwrap();
        assert!(!provider.subreddits.is_empty());
        assert!(provider.subreddits.contains(&"technology".to_string()));
    }

    #[test]
    fn test_extract_metadata() {
        let provider = RedditProvider::new(vec![], None, true).unwrap();
        let content = "submitted by user with 150 points and 42 comments";
        let (score, comments) = provider.extract_metadata_from_content(content);
        assert_eq!(score, Some(150));
        assert_eq!(comments, Some(42));
    }
}
