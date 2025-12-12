//! Comments view
//!
//! Renders threaded comments for HN and Reddit posts

use crate::models::Comment;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

const INDENT_WIDTH: usize = 2;
const MAX_VISIBLE_DEPTH: u32 = 5;

/// Render comments view
pub fn render(
    f: &mut Frame,
    comments: &[Comment],
    selected_idx: usize,
    scroll_offset: usize,
    provider_name: &str,
) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Comments
            Constraint::Length(2), // Help bar
        ])
        .split(size);

    render_header(f, chunks[0], comments, provider_name);
    render_comments(f, chunks[1], comments, selected_idx, scroll_offset);
    render_help(f, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect, comments: &[Comment], provider_name: &str) {
    let total = total_comment_count(comments);
    let title = format!(" Comments ({}) - {} ", total, provider_name);

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

fn render_comments(
    f: &mut Frame,
    area: Rect,
    comments: &[Comment],
    selected_idx: usize,
    scroll_offset: usize,
) {
    // Flatten comments for display
    let flattened = flatten_comments(comments);
    let visible_height = area.height.saturating_sub(2) as usize;

    let list_items: Vec<ListItem> = flattened
        .iter()
        .skip(scroll_offset)
        .take(visible_height)
        .enumerate()
        .map(|(display_idx, (comment, depth))| {
            let actual_idx = scroll_offset + display_idx;
            let is_selected = actual_idx == selected_idx;
            render_comment_line(comment, *depth, is_selected, area.width as usize)
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(list, area);
}

fn render_comment_line(
    comment: &Comment,
    depth: u32,
    is_selected: bool,
    width: usize,
) -> ListItem<'static> {
    let indent = "  ".repeat((depth as usize).min(MAX_VISIBLE_DEPTH as usize) * INDENT_WIDTH);
    let depth_indicator = if depth > 0 { "|" } else { "" };

    let base_style = if is_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let author_style = Style::default().fg(Color::Green);
    let score_style = Style::default().fg(Color::Cyan);
    let time_style = Style::default().fg(Color::DarkGray);

    // Header line: author, score, time
    let score_text = comment
        .score
        .map(|s| format!(" [{}]", s))
        .unwrap_or_default();

    let header = Line::from(vec![
        Span::raw(indent.clone()),
        Span::raw(depth_indicator),
        Span::styled(comment.author.clone(), author_style),
        Span::styled(score_text, score_style),
        Span::styled(format!(" {}", comment.time_ago()), time_style),
    ]);

    // Text preview (truncated)
    let text = comment.text_plain.as_ref().unwrap_or(&comment.text);
    let available_width = width.saturating_sub(indent.len() + 4);
    let text_preview: String = text
        .lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(available_width)
        .collect();

    let text_line = Line::from(vec![
        Span::raw(indent),
        Span::styled(if is_selected { "> " } else { "  " }, base_style),
        Span::styled(text_preview, base_style),
    ]);

    // Combined into a single list item with 2 lines
    ListItem::new(vec![header, text_line])
}

fn render_help(f: &mut Frame, area: Rect) {
    let help = Paragraph::new(Line::from(vec![
        Span::styled("jk/", Style::default().fg(Color::Yellow)),
        Span::raw("Navigate "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(":Expand "),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw(":Collapse "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(":Back"),
    ]))
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(help, area);
}

/// Flatten nested comments into a list with depth info
fn flatten_comments(comments: &[Comment]) -> Vec<(&Comment, u32)> {
    let mut result = Vec::new();
    for comment in comments {
        flatten_comment_recursive(comment, 0, &mut result);
    }
    result
}

fn flatten_comment_recursive<'a>(
    comment: &'a Comment,
    depth: u32,
    result: &mut Vec<(&'a Comment, u32)>,
) {
    if !comment.collapsed {
        result.push((comment, depth));
        if depth < MAX_VISIBLE_DEPTH {
            for reply in &comment.replies {
                flatten_comment_recursive(reply, depth + 1, result);
            }
        }
    } else {
        result.push((comment, depth));
    }
}

/// Count total comments including replies
fn total_comment_count(comments: &[Comment]) -> usize {
    comments.iter().map(|c| c.total_count()).sum()
}
