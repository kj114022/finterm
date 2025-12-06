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
