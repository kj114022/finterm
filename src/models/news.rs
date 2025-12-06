use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a financial news article from Finnhub API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub id: String,
    pub title: String,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub source: String,
    pub author: Option<String>,
    pub url: String,
    pub published_at: DateTime<Utc>,
    pub sentiment: Option<Sentiment>,
    pub tags: Vec<String>,
    pub image_url: Option<String>,
}

/// Sentiment analysis for an article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    pub score: f32,           // -1.0 (negative) to 1.0 (positive)
    pub label: SentimentLabel,
    pub confidence: f32,       // 0.0 to 1.0
}

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
}

/// Response from Finnhub API for news listing
#[derive(Debug, Deserialize)]
pub struct NewsResponse {
    pub articles: Vec<NewsArticle>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

/// Search query parameters
#[derive(Debug, Clone, Serialize)]
pub struct SearchQuery {
    pub query: String,
    pub sources: Option<Vec<String>>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub sentiment: Option<SentimentLabel>,
    pub limit: Option<usize>,
}

impl NewsArticle {
    /// Get a display-friendly time string (e.g., "2 hours ago")
    pub fn time_ago(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.published_at);
        
        if duration.num_seconds() < 60 {
            "just now".to_string()
        } else if duration.num_minutes() < 60 {
            format!("{} minutes ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_days() < 7 {
            format!("{} days ago", duration.num_days())
        } else {
            self.published_at.format("%Y-%m-%d").to_string()
        }
    }
    
    /// Get sentiment color string for UI rendering
    pub fn sentiment_color(&self) -> &str {
        match &self.sentiment {
            Some(s) => match s.label {
                SentimentLabel::Positive => "green",
                SentimentLabel::Negative => "red",
                SentimentLabel::Neutral => "yellow",
            },
            None => "white",
        }
    }
}
