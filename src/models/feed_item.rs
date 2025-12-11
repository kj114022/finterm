use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unified feed item model for all providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedItem {
    /// Unique identifier for this item
    pub id: String,
    /// Provider that supplied this item (e.g., "finnhub", "hackernews", "rss:example.com")
    pub provider_id: String,
    /// Article/story title
    pub title: String,
    /// Short summary or description
    pub summary: Option<String>,
    /// Full content (if available)
    pub content: Option<String>,
    /// Link to the original article
    pub url: Option<String>,
    /// Author name
    pub author: Option<String>,
    /// Source name (e.g., "MarketWatch", "Show HN")
    pub source: String,
    /// Publication timestamp
    pub published_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: FeedItemMetadata,
}

/// Additional metadata for feed items
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeedItemMetadata {
    /// Score/upvotes (HN, Reddit, etc.)
    pub score: Option<i32>,
    /// Comment count
    pub comments: Option<i32>,
    /// Sentiment analysis
    pub sentiment: Option<Sentiment>,
    /// Tags/categories
    pub tags: Vec<String>,
    /// Image URL for thumbnail
    pub image_url: Option<String>,
    /// Provider-specific data stored as JSON
    pub extra: Option<serde_json::Value>,
    /// Full comment thread (loaded on demand)
    pub comments_data: Option<Vec<Comment>>,
    /// Rich link preview
    pub link_preview: Option<LinkPreview>,
    /// Upvote ratio (Reddit-specific, 0.0-1.0)
    pub upvote_ratio: Option<f32>,
    /// Subreddit name (Reddit-specific)
    pub subreddit: Option<String>,
    /// HN item ID for fetching comments
    pub hn_id: Option<u64>,
    /// Reddit post ID for fetching comments
    pub reddit_id: Option<String>,
}

/// A comment in a discussion thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Unique identifier
    pub id: String,
    /// Comment author
    pub author: String,
    /// Comment text/body (may contain HTML/markdown)
    pub text: String,
    /// Plain text version for display
    pub text_plain: Option<String>,
    /// Score/upvotes
    pub score: Option<i32>,
    /// Nested replies
    pub replies: Vec<Comment>,
    /// When the comment was created
    pub created_at: DateTime<Utc>,
    /// Depth in thread (0 = top-level)
    pub depth: u32,
    /// Is this comment collapsed/hidden
    pub collapsed: bool,
}

impl Default for Comment {
    fn default() -> Self {
        Self {
            id: String::new(),
            author: String::new(),
            text: String::new(),
            text_plain: None,
            score: None,
            replies: Vec::new(),
            created_at: Utc::now(),
            depth: 0,
            collapsed: false,
        }
    }
}

impl Comment {
    /// Create a new comment
    pub fn new(id: String, author: String, text: String, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            author,
            text,
            text_plain: None,
            score: None,
            replies: Vec::new(),
            created_at,
            depth: 0,
            collapsed: false,
        }
    }

    /// Count total comments including replies
    pub fn total_count(&self) -> usize {
        1 + self.replies.iter().map(|r| r.total_count()).sum::<usize>()
    }
    
    /// Get time ago string
    pub fn time_ago(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.created_at);
        
        if duration.num_minutes() < 60 {
            format!("{}m", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{}h", duration.num_hours())
        } else {
            format!("{}d", duration.num_days())
        }
    }
}

/// Rich link preview data (Open Graph / meta tags)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkPreview {
    /// Page title (og:title)
    pub title: Option<String>,
    /// Page description (og:description)
    pub description: Option<String>,
    /// Thumbnail image URL (og:image)
    pub image_url: Option<String>,
    /// Site name (og:site_name)
    pub site_name: Option<String>,
    /// Extracted article content snippet
    pub content_snippet: Option<String>,
    /// Favicon URL
    pub favicon_url: Option<String>,
    /// Content type (article, video, etc.)
    pub content_type: Option<String>,
    /// Reading time estimate in minutes
    pub reading_time: Option<u32>,
}

impl Default for LinkPreview {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            image_url: None,
            site_name: None,
            content_snippet: None,
            favicon_url: None,
            content_type: None,
            reading_time: None,
        }
    }
}

/// Sentiment analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    /// Score from -1.0 (negative) to 1.0 (positive)
    pub score: f32,
    /// Human-readable label
    pub label: SentimentLabel,
    /// Confidence level 0.0 to 1.0
    pub confidence: f32,
}

/// Sentiment label categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SentimentLabel {
    Positive,
    Negative,
    Neutral,
}

impl SentimentLabel {
    pub fn as_str(&self) -> &str {
        match self {
            SentimentLabel::Positive => "Positive",
            SentimentLabel::Negative => "Negative",
            SentimentLabel::Neutral => "Neutral",
        }
    }
    
    pub fn color(&self) -> &str {
        match self {
            SentimentLabel::Positive => "green",
            SentimentLabel::Negative => "red",
            SentimentLabel::Neutral => "yellow",
        }
    }
}

impl FeedItem {
    /// Create a new FeedItem with required fields
    pub fn new(
        id: String,
        provider_id: String,
        title: String,
        source: String,
        published_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            provider_id,
            title,
            summary: None,
            content: None,
            url: None,
            author: None,
            source,
            published_at,
            metadata: FeedItemMetadata::default(),
        }
    }
    
    /// Builder method: set summary
    pub fn with_summary(mut self, summary: String) -> Self {
        self.summary = Some(summary);
        self
    }
    
    /// Builder method: set URL
    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }
    
    /// Builder method: set author
    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }
    
    /// Builder method: set content
    pub fn with_content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }
    
    /// Builder method: set metadata
    pub fn with_metadata(mut self, metadata: FeedItemMetadata) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// Get a display-friendly time string (e.g., "2 hours ago")
    pub fn time_ago(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.published_at);
        
        if duration.num_seconds() < 60 {
            "just now".to_string()
        } else if duration.num_minutes() < 60 {
            format!("{}m ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{}h ago", duration.num_hours())
        } else if duration.num_days() < 7 {
            format!("{}d ago", duration.num_days())
        } else {
            self.published_at.format("%Y-%m-%d").to_string()
        }
    }
    
    /// Get sentiment color for UI
    pub fn sentiment_color(&self) -> &str {
        match &self.metadata.sentiment {
            Some(s) => s.label.color(),
            None => "white",
        }
    }
    
    /// Get display string with score if available
    pub fn score_display(&self) -> Option<String> {
        self.metadata.score.map(|s| format!("▲{}", s))
    }
    
    /// Get comments display if available
    pub fn comments_display(&self) -> Option<String> {
        self.metadata.comments.map(|c| format!("💬{}", c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feed_item_builder() {
        let item = FeedItem::new(
            "123".to_string(),
            "test".to_string(),
            "Test Title".to_string(),
            "Test Source".to_string(),
            Utc::now(),
        )
        .with_summary("Test summary".to_string())
        .with_url("https://example.com".to_string());
        
        assert_eq!(item.title, "Test Title");
        assert_eq!(item.summary, Some("Test summary".to_string()));
        assert_eq!(item.url, Some("https://example.com".to_string()));
    }
}
