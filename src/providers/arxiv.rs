//! arXiv provider
//!
//! Fetches latest papers from arXiv RSS feeds

use crate::models::{FeedItem, FeedItemMetadata};
use crate::providers::{FeedProvider, ProviderError, ProviderStatus, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::Client;
use std::time::Duration;

const ARXIV_RSS_BASE: &str = "https://rss.arxiv.org/rss";

/// arXiv category for papers
#[derive(Debug, Clone, Default)]
pub enum ArxivCategory {
    #[default]
    CS, // Computer Science (all)
    CSAI,    // Artificial Intelligence
    CSLG,    // Machine Learning
    CSCL,    // Computation & Language (NLP)
    CSCV,    // Computer Vision
    CSNE,    // Neural & Evolutionary Computing
    Math,    // Mathematics
    Physics, // Physics
    Stat,    // Statistics
}

impl ArxivCategory {
    pub fn as_rss_path(&self) -> &str {
        match self {
            ArxivCategory::CS => "cs",
            ArxivCategory::CSAI => "cs.AI",
            ArxivCategory::CSLG => "cs.LG",
            ArxivCategory::CSCL => "cs.CL",
            ArxivCategory::CSCV => "cs.CV",
            ArxivCategory::CSNE => "cs.NE",
            ArxivCategory::Math => "math",
            ArxivCategory::Physics => "physics",
            ArxivCategory::Stat => "stat",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            ArxivCategory::CS => "Computer Science",
            ArxivCategory::CSAI => "AI",
            ArxivCategory::CSLG => "Machine Learning",
            ArxivCategory::CSCL => "NLP",
            ArxivCategory::CSCV => "Computer Vision",
            ArxivCategory::CSNE => "Neural Computing",
            ArxivCategory::Math => "Mathematics",
            ArxivCategory::Physics => "Physics",
            ArxivCategory::Stat => "Statistics",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "cs.ai" | "ai" => ArxivCategory::CSAI,
            "cs.lg" | "lg" | "ml" | "machine learning" => ArxivCategory::CSLG,
            "cs.cl" | "cl" | "nlp" => ArxivCategory::CSCL,
            "cs.cv" | "cv" | "vision" => ArxivCategory::CSCV,
            "cs.ne" | "ne" | "neural" => ArxivCategory::CSNE,
            "math" | "mathematics" => ArxivCategory::Math,
            "physics" | "phys" => ArxivCategory::Physics,
            "stat" | "statistics" => ArxivCategory::Stat,
            _ => ArxivCategory::CS,
        }
    }
}

/// arXiv paper entry from RSS
#[derive(Debug, Default)]
struct ArxivEntry {
    title: String,
    link: String,
    description: String,
    creator: String,
    pub_date: String,
}

/// arXiv provider
pub struct ArxivProvider {
    client: Client,
    category: ArxivCategory,
    enabled: bool,
}

impl ArxivProvider {
    /// Create a new arXiv provider
    pub fn new(category: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .user_agent("FinTerm/0.3.0 (https://github.com/kj114022/finterm)")
            .build()
            .map_err(|e| ProviderError::Other(e.to_string()))?;

        Ok(Self {
            client,
            category: category
                .map(|c| ArxivCategory::from_str(&c))
                .unwrap_or_default(),
            enabled: true,
        })
    }

    /// Set category
    pub fn set_category(&mut self, category: ArxivCategory) {
        self.category = category;
    }

    /// Fetch RSS feed
    async fn fetch_feed(&self) -> Result<String> {
        let url = format!("{}/{}", ARXIV_RSS_BASE, self.category.as_rss_path());

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

    /// Parse RSS feed into entries
    fn parse_feed(&self, xml: &str) -> Result<Vec<ArxivEntry>> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let mut entries = Vec::new();
        let mut current_entry: Option<ArxivEntry> = None;
        let mut current_tag = String::new();
        let mut in_item = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    current_tag = tag_name.clone();

                    if tag_name == "item" {
                        in_item = true;
                        current_entry = Some(ArxivEntry::default());
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag_name == "item" {
                        if let Some(entry) = current_entry.take() {
                            if !entry.title.is_empty() {
                                entries.push(entry);
                            }
                        }
                        in_item = false;
                    }
                }
                Ok(Event::Text(e)) => {
                    if in_item {
                        if let Some(ref mut entry) = current_entry {
                            let text = e.unescape().unwrap_or_default().to_string();
                            match current_tag.as_str() {
                                "title" => entry.title = text,
                                "link" => entry.link = text,
                                "description" => entry.description = text,
                                "dc:creator" => entry.creator = text,
                                "pubDate" => entry.pub_date = text,
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ProviderError::Parse(format!("XML error: {}", e)));
                }
                _ => {}
            }
        }

        Ok(entries)
    }

    /// Convert entry to FeedItem
    fn convert_to_feed_item(&self, entry: ArxivEntry) -> FeedItem {
        let published_at = DateTime::parse_from_rfc2822(&entry.pub_date)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        // Clean title (remove newlines/extra spaces)
        let title = entry
            .title
            .lines()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join(" ");

        // Extract arXiv ID from link
        let arxiv_id = entry
            .link
            .split('/')
            .next_back()
            .unwrap_or("unknown")
            .to_string();

        let metadata = FeedItemMetadata {
            tags: vec![
                self.category.display_name().to_string(),
                "paper".to_string(),
            ],
            ..Default::default()
        };

        let source = format!("arXiv:{}", self.category.display_name());

        // Clean description (HTML to text)
        let summary = html2text::from_read(entry.description.as_bytes(), 200);

        let mut item = FeedItem::new(arxiv_id, self.id().to_string(), title, source, published_at)
            .with_metadata(metadata)
            .with_url(entry.link);

        if !entry.creator.is_empty() {
            item = item.with_author(entry.creator);
        }

        if !summary.trim().is_empty() {
            item = item.with_summary(summary);
        }

        item
    }
}

#[async_trait]
impl FeedProvider for ArxivProvider {
    fn id(&self) -> &str {
        "arxiv"
    }

    fn name(&self) -> &str {
        "arXiv"
    }

    fn description(&self) -> &str {
        "Open-access research papers in physics, mathematics, and computer science"
    }

    fn icon(&self) -> &str {
        "[aX]"
    }

    fn status(&self) -> ProviderStatus {
        if self.enabled {
            ProviderStatus::Ready
        } else {
            ProviderStatus::Disabled
        }
    }

    fn categories(&self) -> Vec<&str> {
        vec![
            "cs", "cs.ai", "cs.lg", "cs.cl", "cs.cv", "math", "physics", "stat",
        ]
    }

    async fn fetch_items(&self, limit: usize) -> Result<Vec<FeedItem>> {
        let xml = self.fetch_feed().await?;
        let entries = self.parse_feed(&xml)?;

        let items: Vec<FeedItem> = entries
            .into_iter()
            .take(limit)
            .map(|e| self.convert_to_feed_item(e))
            .collect();

        Ok(items)
    }

    fn supports_search(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_ready() {
        let provider = ArxivProvider::new(None).unwrap();
        assert_eq!(provider.status(), ProviderStatus::Ready);
        assert_eq!(provider.id(), "arxiv");
        assert_eq!(provider.name(), "arXiv");
    }

    #[test]
    fn test_category_parsing() {
        assert!(matches!(
            ArxivCategory::from_str("cs.ai"),
            ArxivCategory::CSAI
        ));
        assert!(matches!(ArxivCategory::from_str("ml"), ArxivCategory::CSLG));
        assert!(matches!(
            ArxivCategory::from_str("nlp"),
            ArxivCategory::CSCL
        ));
    }

    #[tokio::test]
    async fn test_fetch_items() {
        let provider = ArxivProvider::new(Some("cs.ai".to_string())).unwrap();
        let result = provider.fetch_items(5).await;
        // Note: This test requires network access
        if result.is_ok() {
            let items = result.unwrap();
            println!("Fetched {} arXiv papers", items.len());
        }
    }
}
