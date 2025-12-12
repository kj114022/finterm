//! Quick test to verify Reddit RSS fetching works

use finterm::providers::{FeedProvider, RedditProvider};

#[tokio::main]
async fn main() {
    println!("Testing Reddit RSS Provider...\n");

    // Create provider with default subreddits
    let provider = RedditProvider::new(
        vec!["rust".to_string(), "programming".to_string()],
        Some("hot".to_string()),
        true,
    )
    .expect("Failed to create Reddit provider");

    println!("Provider: {} ({})", provider.name(), provider.id());
    println!("Status: {:?}", provider.status());
    println!("Categories: {:?}", provider.categories());
    println!("\nFetching 10 items...\n");

    match provider.fetch_items(10).await {
        Ok(items) => {
            println!("Successfully fetched {} items:\n", items.len());
            for (i, item) in items.iter().enumerate() {
                println!("{}. [{}] {}", i + 1, item.source, item.title);
                if let Some(author) = &item.author {
                    println!("   Author: {}", author);
                }
                if let Some(score) = item.metadata.score {
                    print!("   Score: {} ", score);
                }
                if let Some(comments) = item.metadata.comments {
                    print!("Comments: {}", comments);
                }
                println!("\n   Published: {}", item.time_ago());
                if let Some(url) = &item.url {
                    println!("   URL: {}", url);
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error fetching items: {}", e);
        }
    }
}
