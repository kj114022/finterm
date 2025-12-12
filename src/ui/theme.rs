//! Theme module
//!
//! Bloomberg-inspired color scheme and consistent styling

use ratatui::style::{Color, Modifier, Style};

/// Provider-specific brand colors
pub struct ProviderColors;

impl ProviderColors {
    pub fn hackernews() -> Color {
        Color::Rgb(255, 102, 0) // HN Orange #FF6600
    }

    pub fn reddit() -> Color {
        Color::Rgb(255, 69, 0) // Reddit Orange-red #FF4500
    }

    pub fn finnhub() -> Color {
        Color::Rgb(0, 102, 204) // Finnhub Blue #0066CC
    }

    pub fn cratesio() -> Color {
        Color::Rgb(247, 76, 0) // Rust Orange #F74C00
    }

    pub fn for_provider(provider_id: &str) -> Color {
        match provider_id {
            "hackernews" => Self::hackernews(),
            "reddit" => Self::reddit(),
            "finnhub" => Self::finnhub(),
            "cratesio" => Self::cratesio(),
            _ => Color::Cyan,
        }
    }
}

/// Bloomberg-inspired dark theme
pub struct Theme;

impl Theme {
    // Background colors
    pub fn bg_primary() -> Color {
        Color::Rgb(17, 17, 17) // Almost black
    }

    pub fn bg_secondary() -> Color {
        Color::Rgb(26, 26, 26) // Slightly lighter
    }

    pub fn bg_highlight() -> Color {
        Color::Rgb(38, 38, 38) // Selection background
    }

    // Text colors
    pub fn text_primary() -> Color {
        Color::Rgb(229, 229, 229) // Off-white
    }

    pub fn text_secondary() -> Color {
        Color::Rgb(128, 128, 128) // Gray
    }

    pub fn text_muted() -> Color {
        Color::Rgb(85, 85, 85) // Dark gray
    }

    // Accent colors
    pub fn accent_primary() -> Color {
        Color::Rgb(255, 136, 0) // Bloomberg orange
    }

    pub fn accent_secondary() -> Color {
        Color::Cyan
    }

    // Semantic colors
    pub fn positive() -> Color {
        Color::Rgb(0, 200, 83) // Green for positive
    }

    pub fn negative() -> Color {
        Color::Rgb(207, 102, 121) // Red for negative
    }

    pub fn warning() -> Color {
        Color::Yellow
    }

    // Border colors
    pub fn border_default() -> Color {
        Color::Rgb(51, 51, 51)
    }

    pub fn border_focus() -> Color {
        Color::Rgb(255, 136, 0) // Orange when focused
    }

    // Styles
    pub fn style_header() -> Style {
        Style::default()
            .fg(Self::accent_primary())
            .add_modifier(Modifier::BOLD)
    }

    pub fn style_selected() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn style_title() -> Style {
        Style::default().fg(Self::text_primary())
    }

    pub fn style_meta() -> Style {
        Style::default().fg(Self::text_secondary())
    }

    pub fn style_muted() -> Style {
        Style::default().fg(Self::text_muted())
    }

    pub fn style_score() -> Style {
        Style::default().fg(Self::positive())
    }

    pub fn style_comments() -> Style {
        Style::default().fg(Color::Cyan)
    }

    pub fn style_author() -> Style {
        Style::default().fg(Color::Rgb(150, 150, 255)) // Light blue
    }

    pub fn style_time() -> Style {
        Style::default().fg(Self::text_muted())
    }

    pub fn style_border() -> Style {
        Style::default().fg(Self::border_default())
    }

    pub fn style_border_focus() -> Style {
        Style::default().fg(Self::border_focus())
    }
}
