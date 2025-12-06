use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Hacker News story/item type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Story,
    Comment,
    Job,
    Poll,
    Pollopt,
}

/// Represents a Hacker News story or comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnItem {
    pub id: u64,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    pub by: Option<String>,        // Author username
    pub time: i64,                  // Unix timestamp
    pub text: Option<String>,       // Comment text (HTML)
    pub dead: Option<bool>,
    pub deleted: Option<bool>,
    pub parent: Option<u64>,        // Parent item ID
    pub kids: Option<Vec<u64>>,     // Child comment IDs
    pub url: Option<String>,        // Story URL
    pub score: Option<i32>,
    pub title: Option<String>,
    pub descendants: Option<i32>,   // Total comment count
}

/// Hacker News story with fetched content and comments
#[derive(Debug, Clone)]
pub struct HnStory {
    pub item: HnItem,
    pub content: Option<String>,    // Fetched article content
    pub comments: Vec<HnComment>,
}

/// Represents a comment with its thread
#[derive(Debug, Clone)]
pub struct HnComment {
    pub item: HnItem,
    pub children: Vec<HnComment>,
    pub collapsed: bool,
    pub depth: usize,
}

/// Category for fetching HN stories
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HnCategory {
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
}

impl HnItem {
    /// Check if item is deleted or dead
    pub fn is_valid(&self) -> bool {
        !self.deleted.unwrap_or(false) && !self.dead.unwrap_or(false)
    }
    
    /// Get DateTime from Unix timestamp
    pub fn datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.time, 0).unwrap_or_else(Utc::now)
    }
    
    /// Get display-friendly time string
    pub fn time_ago(&self) -> String {
        let now = Utc::now();
        let dt = self.datetime();
        let duration = now.signed_duration_since(dt);
        
        if duration.num_seconds() < 60 {
            "just now".to_string()
        } else if duration.num_minutes() < 60 {
            format!("{} minutes ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_days() < 7 {
            format!("{} days ago", duration.num_days())
        } else {
            dt.format("%Y-%m-%d").to_string()
        }
    }
    
    /// Get plain text from HTML content
    pub fn plain_text(&self) -> Option<String> {
        self.text.as_ref().map(|html| {
            html2text::from_read(html.as_bytes(), 80)
        })
    }
}

impl HnComment {
    /// Create a new comment with depth tracking
    pub fn new(item: HnItem, depth: usize) -> Self {
        Self {
            item,
            children: Vec::new(),
            collapsed: depth > 2, // Auto-collapse deep threads
            depth,
        }
    }
    
    /// Toggle collapse state
    pub fn toggle_collapse(&mut self) {
        self.collapsed = !self.collapsed;
    }
    
    /// Count total descendants (including nested)
    pub fn count_descendants(&self) -> usize {
        self.children.len() + self.children.iter().map(|c| c.count_descendants()).sum::<usize>()
    }
}

impl HnStory {
    /// Create a new story from an item
    pub fn new(item: HnItem) -> Self {
        Self {
            item,
            content: None,
            comments: Vec::new(),
        }
    }
    
    /// Get display title
    pub fn title(&self) -> &str {
        self.item.title.as_deref().unwrap_or("(no title)")
    }
    
    /// Get comment count
    pub fn comment_count(&self) -> i32 {
        self.item.descendants.unwrap_or(0)
    }
}
