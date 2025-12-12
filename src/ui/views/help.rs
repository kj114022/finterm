//! Help view
//!
//! Displays keybinding reference

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the help view
pub fn render(f: &mut Frame, bindings: &[(&str, &str)]) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Help content
        ])
        .split(size);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "📖 Help",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - Keybindings Reference"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(header, chunks[0]);

    // Help content
    let mut lines = Vec::new();

    for (key, desc) in bindings {
        if key.is_empty() {
            lines.push(Line::from(""));
        } else if desc.is_empty() {
            // Section header
            lines.push(Line::from(vec![Span::styled(
                *key,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(format!("{:18}", key), Style::default().fg(Color::Green)),
                Span::raw(*desc),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press Esc or ? to close",
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::ITALIC),
    )));

    let help_content = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(help_content, chunks[1]);
}
