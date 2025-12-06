//! Article view
//! 
//! Renders a single article/feed item in detail

use crate::models::feed_item::{FeedItem, SentimentLabel};
use crate::utils::wrap_text;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Render a FeedItem in article view
pub fn render_feed_item(f: &mut Frame, item: &FeedItem, scroll_offset: usize) {
    let size = f.size();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header/Title
            Constraint::Length(2),  // Metadata
            Constraint::Min(5),     // Content
            Constraint::Length(2),  // Help bar
        ])
        .split(size);
    
    // Render header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            &item.title,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan)));
    
    f.render_widget(header, chunks[0]);
    
    // Render metadata
    let mut meta_parts = vec![
        Span::styled("Source: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&item.source, Style::default().fg(Color::White)),
        Span::raw(" | "),
        Span::styled(item.time_ago(), Style::default().fg(Color::DarkGray)),
    ];
    
    if let Some(author) = &item.author {
        meta_parts.push(Span::raw(" | by "));
        meta_parts.push(Span::styled(author, Style::default().fg(Color::Yellow)));
    }
    
    if let Some(sentiment) = &item.metadata.sentiment {
        meta_parts.push(Span::raw(" | "));
        let sentiment_style = match sentiment.label {
            SentimentLabel::Positive => Style::default().fg(Color::Green),
            SentimentLabel::Negative => Style::default().fg(Color::Red),
            SentimentLabel::Neutral => Style::default().fg(Color::Yellow),
        };
        meta_parts.push(Span::styled(sentiment.label.as_str(), sentiment_style));
    }
    
    if let Some(score) = item.metadata.score {
        meta_parts.push(Span::raw(" | "));
        meta_parts.push(Span::styled(format!("▲{}", score), Style::default().fg(Color::Green)));
    }
    
    if let Some(comments) = item.metadata.comments {
        meta_parts.push(Span::raw(" "));
        meta_parts.push(Span::styled(format!("💬{}", comments), Style::default().fg(Color::DarkGray)));
    }
    
    let meta_paragraph = Paragraph::new(Line::from(meta_parts))
        .style(Style::default().fg(Color::Gray));
    
    f.render_widget(meta_paragraph, chunks[1]);
    
    // Render content with scrolling
    let content_text = item.content.as_ref()
        .or(item.summary.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| {
            if let Some(url) = &item.url {
                format!("No content preview available.\n\nOpen in browser: {}", url)
            } else {
                "No content available".to_string()
            }
        });
    
    let content_width = chunks[2].width.saturating_sub(4) as usize;
    let wrapped_lines = wrap_text(&content_text, content_width.max(20));
    
    let visible_lines: Vec<Line> = wrapped_lines
        .iter()
        .skip(scroll_offset)
        .take(chunks[2].height.saturating_sub(2) as usize)
        .map(|s| Line::from(s.clone()))
        .collect();
    
    let total_lines = wrapped_lines.len();
    let visible_count = visible_lines.len();
    let scroll_indicator = if total_lines > visible_count {
        format!(" [{}/{} lines] ", scroll_offset + 1, total_lines)
    } else {
        String::new()
    };
    
    let content_paragraph = Paragraph::new(visible_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(scroll_indicator))
        .wrap(Wrap { trim: false });
    
    f.render_widget(content_paragraph, chunks[2]);
    
    // Render help bar
    let help = Paragraph::new(Line::from(vec![
        Span::styled("↑↓/jk", Style::default().fg(Color::Yellow)),
        Span::raw(":Scroll "),
        Span::styled("[/]", Style::default().fg(Color::Yellow)),
        Span::raw(":Prev/Next "),
        Span::styled("o", Style::default().fg(Color::Yellow)),
        Span::raw(":Open "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(":Back"),
    ]))
    .block(Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray)));
    
    f.render_widget(help, chunks[3]);
}

// Keep old render function for backward compatibility (deprecated)
#[deprecated(note = "Use render_feed_item instead")]
#[allow(dead_code)]
pub fn render(f: &mut Frame, _app: &()) {
    // This function is deprecated
    let msg = Paragraph::new("Article view deprecated - use render_feed_item")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(msg, f.size());
}
