//! Dashboard view
//!
//! Bloomberg-style multi-panel layout with feed list and preview

use crate::models::FeedItem;
use crate::ui::theme::Theme;
use crate::utils::parser::truncate;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Render dashboard with split layout
pub fn render(
    f: &mut Frame,
    provider_name: &str,
    provider_icon: &str,
    provider_color: Color,
    items: &[FeedItem],
    selected_idx: usize,
    status_message: Option<&str>,
    loading: bool,
) {
    let size = f.size();

    // Main vertical layout
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Content area
            Constraint::Length(3), // Status bar
        ])
        .split(size);

    render_header(
        f,
        main_chunks[0],
        provider_name,
        provider_icon,
        provider_color,
        items.len(),
    );

    // Split content area into list and preview
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Feed list
            Constraint::Percentage(50), // Preview panel
        ])
        .split(main_chunks[1]);

    render_feed_list(f, content_chunks[0], items, selected_idx, provider_color);
    render_preview_panel(f, content_chunks[1], items, selected_idx);
    render_status_bar(f, main_chunks[2], status_message, loading);
}

fn render_header(f: &mut Frame, area: Rect, name: &str, icon: &str, color: Color, count: usize) {
    let title = format!(" {} {} ", icon, name);
    let count_text = format!(" {} items ", count);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled("|", Style::default().fg(Theme::border_default())),
        Span::styled(count_text, Theme::style_meta()),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color)),
    );

    f.render_widget(header, area);
}

fn render_feed_list(
    f: &mut Frame,
    area: Rect,
    items: &[FeedItem],
    selected_idx: usize,
    accent: Color,
) {
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

            render_feed_item(item, is_selected, area.width as usize, accent)
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if true {
                Style::default().fg(accent)
            } else {
                Theme::style_border()
            })
            .title(" Feed "),
    );

    f.render_widget(list, area);
}

fn render_feed_item(
    item: &FeedItem,
    is_selected: bool,
    width: usize,
    _accent: Color,
) -> ListItem<'static> {
    let style = if is_selected {
        Theme::style_selected()
    } else {
        Theme::style_title()
    };

    let prefix = if is_selected { "> " } else { "  " };
    let title = truncate(&item.title, width.saturating_sub(25));

    let mut line1_spans = vec![
        Span::styled(prefix.to_string(), style),
        Span::styled(title, style),
    ];

    // Add score if available
    if let Some(score) = item.score_display() {
        line1_spans.push(Span::raw(" "));
        line1_spans.push(Span::styled(score, Theme::style_score()));
    }

    // Add comments if available
    if let Some(comments) = item.comments_display() {
        line1_spans.push(Span::raw(" "));
        line1_spans.push(Span::styled(comments, Theme::style_comments()));
    }

    // Second line: metadata
    let meta_prefix = if is_selected { "  " } else { "  " };
    let mut line2_spans = vec![
        Span::raw(meta_prefix),
        Span::styled(item.source.clone(), Theme::style_muted()),
    ];

    if let Some(author) = &item.author {
        line2_spans.push(Span::styled(
            format!(" by {}", author),
            Theme::style_author(),
        ));
    }

    line2_spans.push(Span::styled(
        format!(" {}", item.time_ago()),
        Theme::style_time(),
    ));

    ListItem::new(vec![Line::from(line1_spans), Line::from(line2_spans)])
}

fn render_preview_panel(f: &mut Frame, area: Rect, items: &[FeedItem], selected_idx: usize) {
    let item = items.get(selected_idx);

    match item {
        Some(item) => render_item_preview(f, area, item),
        None => render_empty_preview(f, area),
    }
}

fn render_item_preview(f: &mut Frame, area: Rect, item: &FeedItem) {
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(2), // Metadata
            Constraint::Min(5),    // Content
            Constraint::Length(2), // Actions hint
        ])
        .split(area);

    // Block around everything
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::style_border())
        .title(" Preview ");
    f.render_widget(block, area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        &item.title,
        Theme::style_header(),
    )]))
    .wrap(Wrap { trim: true });
    f.render_widget(title, inner_chunks[0]);

    // Metadata
    let mut meta_spans = vec![Span::styled(&item.source, Style::default().fg(Color::Cyan))];

    if let Some(author) = &item.author {
        meta_spans.push(Span::styled(
            format!(" | {}", author),
            Theme::style_author(),
        ));
    }

    meta_spans.push(Span::styled(
        format!(" | {}", item.time_ago()),
        Theme::style_time(),
    ));

    if let Some(score) = item.metadata.score {
        meta_spans.push(Span::styled(
            format!(" | {} pts", score),
            Theme::style_score(),
        ));
    }

    if let Some(comments) = item.metadata.comments {
        meta_spans.push(Span::styled(
            format!(" | {} comments", comments),
            Theme::style_comments(),
        ));
    }

    let meta = Paragraph::new(Line::from(meta_spans));
    f.render_widget(meta, inner_chunks[1]);

    // Content preview
    let content = item
        .summary
        .as_ref()
        .or(item.content.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("No preview available. Press Enter to view full article.");

    let content_para = Paragraph::new(content)
        .style(Theme::style_meta())
        .wrap(Wrap { trim: true });
    f.render_widget(content_para, inner_chunks[2]);

    // Actions hint
    let actions = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(":Open "),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw(":Comments "),
        Span::styled("o", Style::default().fg(Color::Yellow)),
        Span::raw(":Browser"),
    ]))
    .style(Theme::style_muted());
    f.render_widget(actions, inner_chunks[3]);
}

fn render_empty_preview(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::style_border())
        .title(" Preview ");

    let empty = Paragraph::new("No item selected")
        .style(Theme::style_muted())
        .block(block);

    f.render_widget(empty, area);
}

fn render_status_bar(f: &mut Frame, area: Rect, message: Option<&str>, loading: bool) {
    let status = if loading {
        "Loading...".to_string()
    } else {
        message
            .unwrap_or("jk:Navigate Enter:Open c:Comments o:Browser ?:Help q:Quit")
            .to_string()
    };

    let loading_indicator = if loading { "[*] " } else { "" };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(loading_indicator, Style::default().fg(Color::Yellow)),
        Span::styled(status, Theme::style_muted()),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::style_border()),
    );

    f.render_widget(footer, area);
}
