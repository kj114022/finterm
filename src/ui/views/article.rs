//! Article view
//! 
//! Renders a single article/feed item in detail

use crate::models::feed_item::{FeedItem, SentimentLabel};
use crate::utils::wrap_text;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
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
            Constraint::Length(1),  // Reading progress bar
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
    
    // Render metadata with link preview info
    render_metadata(f, chunks[1], item);
    
    // Render content with scrolling
    let total_lines = render_content(f, chunks[2], item, scroll_offset);
    
    // Render reading progress bar
    render_progress_bar(f, chunks[3], scroll_offset, total_lines, chunks[2].height as usize);
    
    // Render help bar with 'c' for comments
    render_help_bar(f, chunks[4]);
}

fn render_metadata(f: &mut Frame, area: Rect, item: &FeedItem) {
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
        meta_parts.push(Span::styled(format!("{} pts", score), Style::default().fg(Color::Green)));
    }
    
    if let Some(comments) = item.metadata.comments {
        meta_parts.push(Span::raw(" | "));
        meta_parts.push(Span::styled(format!("{} comments", comments), Style::default().fg(Color::Cyan)));
    }
    
    // Show link preview site name if available
    if let Some(preview) = &item.metadata.link_preview {
        if let Some(site_name) = &preview.site_name {
            meta_parts.push(Span::raw(" | "));
            meta_parts.push(Span::styled(site_name, Style::default().fg(Color::Magenta)));
        }
        if let Some(reading_time) = preview.reading_time {
            meta_parts.push(Span::styled(format!(" ~{}min", reading_time), Style::default().fg(Color::DarkGray)));
        }
    }
    
    let meta_paragraph = Paragraph::new(Line::from(meta_parts))
        .style(Style::default().fg(Color::Gray));
    
    f.render_widget(meta_paragraph, area);
}

fn render_content(f: &mut Frame, area: Rect, item: &FeedItem, scroll_offset: usize) -> usize {
    let content_text = item.content.as_ref()
        .or(item.summary.as_ref()).cloned()
        .unwrap_or_else(|| {
            if let Some(url) = &item.url {
                format!("No content preview available.\n\nOpen in browser: {}", url)
            } else {
                "No content available".to_string()
            }
        });
    
    let content_width = area.width.saturating_sub(4) as usize;
    let wrapped_lines = wrap_text(&content_text, content_width.max(20));
    let total_lines = wrapped_lines.len();
    
    let visible_lines: Vec<Line> = wrapped_lines
        .iter()
        .skip(scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|s| Line::from(s.clone()))
        .collect();
    
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
    
    f.render_widget(content_paragraph, area);
    total_lines
}

fn render_progress_bar(f: &mut Frame, area: Rect, scroll_offset: usize, total_lines: usize, visible_height: usize) {
    if total_lines == 0 {
        return;
    }
    
    let progress = if total_lines <= visible_height {
        100
    } else {
        let max_scroll = total_lines.saturating_sub(visible_height);
        ((scroll_offset as f64 / max_scroll as f64) * 100.0).min(100.0) as u16
    };
    
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .ratio(progress as f64 / 100.0)
        .label(format!("{}%", progress));
    
    f.render_widget(gauge, area);
}

fn render_help_bar(f: &mut Frame, area: Rect) {
    let help = Paragraph::new(Line::from(vec![
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::raw(":Scroll "),
        Span::styled("[/]", Style::default().fg(Color::Yellow)),
        Span::raw(":Prev/Next "),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw(":Comments "),
        Span::styled("o", Style::default().fg(Color::Yellow)),
        Span::raw(":Open "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(":Back"),
    ]))
    .block(Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray)));
    
    f.render_widget(help, area);
}
