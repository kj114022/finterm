//! Single Feed view
//!
//! Displays items from a single feed provider

use crate::models::FeedItem;
use crate::utils::parser::truncate;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render a single feed view
pub fn render(
    f: &mut Frame,
    provider_name: &str,
    provider_icon: &str,
    items: &[FeedItem],
    selected_idx: usize,
    status_message: Option<&str>,
    loading: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Items list
            Constraint::Length(3), // Status bar
        ])
        .split(f.size());

    render_header(f, chunks[0], provider_name, provider_icon, items.len());
    render_items(f, chunks[1], items, selected_idx);
    render_status(f, chunks[2], status_message, loading);
}

fn render_header(f: &mut Frame, area: Rect, name: &str, icon: &str, count: usize) {
    let title = format!("{} {} ({} items)", icon, name, count);

    let header = Paragraph::new(Line::from(vec![Span::styled(
        title,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(header, area);
}

fn render_items(f: &mut Frame, area: Rect, items: &[FeedItem], selected_idx: usize) {
    let visible_height = area.height.saturating_sub(2) as usize;

    // Calculate scroll offset
    let scroll_offset = if selected_idx >= visible_height {
        selected_idx - visible_height + 1
    } else {
        0
    };

    let list_items: Vec<ListItem> = items
        .iter()
        .skip(scroll_offset)
        .take(visible_height)
        .enumerate()
        .map(|(display_idx, item)| {
            let actual_idx = scroll_offset + display_idx;
            let is_selected = actual_idx == selected_idx;

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if is_selected { "▸ " } else { "  " };
            let title = truncate(&item.title, (area.width as usize).saturating_sub(30));

            let mut spans = vec![Span::styled(prefix, style), Span::styled(title, style)];

            // Add score if available
            if let Some(score) = item.score_display() {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(score, Style::default().fg(Color::Green)));
            }

            // Add comments if available
            if let Some(comments) = item.comments_display() {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(comments, Style::default().fg(Color::DarkGray)));
            }

            // Add time
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                item.time_ago(),
                Style::default().fg(Color::DarkGray),
            ));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(list, area);
}

fn render_status(f: &mut Frame, area: Rect, message: Option<&str>, loading: bool) {
    let status = if loading {
        "Loading...".to_string()
    } else {
        message
            .unwrap_or("Press Enter to open, Esc to go back, ? for help")
            .to_string()
    };

    let footer = Paragraph::new(Line::from(vec![Span::styled(
        status,
        Style::default().fg(Color::DarkGray),
    )]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(footer, area);
}
