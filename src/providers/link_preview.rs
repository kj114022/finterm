//! Link Preview Provider
//!
//! Fetches and parses Open Graph metadata from URLs

use crate::models::LinkPreview;
use reqwest::Client;
use std::time::Duration;

/// Fetch link preview data from a URL
pub async fn fetch_link_preview(client: &Client, url: &str) -> Option<LinkPreview> {
    let response = client
        .get(url)
        .timeout(Duration::from_secs(5))
        .header("User-Agent", "Mozilla/5.0 (compatible; FinTerm/0.3.0)")
        .send()
        .await
        .ok()?;
    
    let html = response.text().await.ok()?;
    parse_open_graph(&html)
}

/// Parse Open Graph metadata from HTML
fn parse_open_graph(html: &str) -> Option<LinkPreview> {
    let mut preview = LinkPreview::default();
    
    // Extract og:title
    preview.title = extract_meta_content(html, "og:title")
        .or_else(|| extract_tag_content(html, "title"));
    
    // Extract og:description
    preview.description = extract_meta_content(html, "og:description")
        .or_else(|| extract_meta_content(html, "description"));
    
    // Extract og:image
    preview.image_url = extract_meta_content(html, "og:image");
    
    // Extract og:site_name
    preview.site_name = extract_meta_content(html, "og:site_name");
    
    // Extract og:type
    preview.content_type = extract_meta_content(html, "og:type");
    
    // Estimate reading time from content length
    let word_count = html.split_whitespace().count();
    if word_count > 100 {
        preview.reading_time = Some((word_count / 200).max(1) as u32);
    }
    
    // Extract content snippet from first paragraph
    preview.content_snippet = extract_first_paragraph(html);
    
    if preview.title.is_some() || preview.description.is_some() {
        Some(preview)
    } else {
        None
    }
}

/// Extract meta tag content by property
fn extract_meta_content(html: &str, property: &str) -> Option<String> {
    // Look for <meta property="og:..." content="...">
    let pattern1 = format!(r#"property="{}""#, property);
    let pattern2 = format!(r#"name="{}""#, property);
    
    for pattern in [pattern1, pattern2] {
        if let Some(pos) = html.find(&pattern) {
            let chunk = &html[pos..];
            if let Some(content_start) = chunk.find("content=\"") {
                let content_chunk = &chunk[content_start + 9..];
                if let Some(end) = content_chunk.find('"') {
                    let content = &content_chunk[..end];
                    if !content.is_empty() {
                        return Some(decode_html_entities(content));
                    }
                }
            }
        }
    }
    None
}

/// Extract content from a tag like <title>...</title>
fn extract_tag_content(html: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}", tag);
    let end_tag = format!("</{}>", tag);
    
    if let Some(start) = html.find(&start_tag) {
        let after_start = &html[start..];
        if let Some(close_bracket) = after_start.find('>') {
            let content_start = &after_start[close_bracket + 1..];
            if let Some(end) = content_start.find(&end_tag) {
                let content = content_start[..end].trim();
                if !content.is_empty() {
                    return Some(decode_html_entities(content));
                }
            }
        }
    }
    None
}

/// Extract first paragraph of content
fn extract_first_paragraph(html: &str) -> Option<String> {
    if let Some(start) = html.find("<p") {
        let after_p = &html[start..];
        if let Some(close) = after_p.find('>') {
            let content = &after_p[close + 1..];
            if let Some(end) = content.find("</p>") {
                let text = &content[..end];
                let plain = html2text::from_read(text.as_bytes(), 200);
                let trimmed = plain.trim();
                if !trimmed.is_empty() && trimmed.len() > 20 {
                    return Some(trimmed.chars().take(300).collect());
                }
            }
        }
    }
    None
}

/// Decode basic HTML entities
fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}
