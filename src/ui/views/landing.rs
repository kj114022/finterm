//! Landing page view
//!
//! Initial screen where users can select which feed source to view

use crate::providers::ProviderRegistry;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render the landing page
pub fn render(f: &mut Frame, registry: &ProviderRegistry, selected_idx: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(8), // Header
            Constraint::Min(10),   // Provider list
            Constraint::Length(3), // Footer
        ])
        .split(f.size());

    render_header(f, chunks[0]);
    render_provider_list(f, chunks[1], registry, selected_idx);
    render_footer(f, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect) {
    let header_text = vec![
        Line::from(vec![
            Span::styled("📰 ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "FinTerm",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" v0.2.0", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Terminal News Aggregator",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Select a feed source or press 'A' for All",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(Block::default());

    f.render_widget(header, area);
}

fn render_provider_list(
    f: &mut Frame,
    area: Rect,
    registry: &ProviderRegistry,
    selected_idx: usize,
) {
    let summaries = registry.status_summary();

    let items: Vec<ListItem> = summaries
        .iter()
        .enumerate()
        .map(|(i, summary)| {
            let prefix = format!("[{}]", i + 1);
            let status_icon = summary.status_indicator();

            let style = if i == selected_idx {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", prefix), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} ", summary.icon), style),
                Span::styled(summary.name.to_string(), style),
                Span::raw(" - "),
                Span::styled(&summary.description, Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled(
                    status_icon,
                    match &summary.status {
                        crate::providers::ProviderStatus::Ready => {
                            Style::default().fg(Color::Green)
                        }
                        crate::providers::ProviderStatus::NeedsConfig => {
                            Style::default().fg(Color::Yellow)
                        }
                        crate::providers::ProviderStatus::Disabled => {
                            Style::default().fg(Color::DarkGray)
                        }
                        crate::providers::ProviderStatus::Error(_) => {
                            Style::default().fg(Color::Red)
                        }
                    },
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    // Add "All" option
    let all_style = if selected_idx == summaries.len() {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let mut all_items = items;
    all_items.push(ListItem::new(Line::from(""))); // Spacer
    all_items.push(ListItem::new(Line::from(vec![
        Span::styled("[A] ", Style::default().fg(Color::DarkGray)),
        Span::styled("🌐 ", all_style),
        Span::styled("All Sources", all_style),
        Span::raw(" - "),
        Span::styled(
            "Combined dashboard view",
            Style::default().fg(Color::DarkGray),
        ),
    ])));

    let list = List::new(all_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Feed Sources "),
    );

    f.render_widget(list, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let footer_text = Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" Navigate  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" Select  "),
        Span::styled("1-9", Style::default().fg(Color::Yellow)),
        Span::raw(" Quick select  "),
        Span::styled("?", Style::default().fg(Color::Yellow)),
        Span::raw(" Help  "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit"),
    ]);

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    f.render_widget(footer, area);
}
