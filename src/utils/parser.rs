/// Parse HTML content and extract clean, readable text  
pub fn extract_readable_text(html: &str) -> String {
    html2text::from_read(html.as_bytes(), 80)
}

/// Clean and normalize text
pub fn clean_text(text: &str) -> String {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Truncate text to a maximum length with ellipsis (Unicode-safe)
pub fn truncate(text: &str, max_len: usize) -> String {
    let char_count: usize = text.chars().count();
    if char_count <= max_len {
        text.to_string()
    } else {
        let truncate_to = max_len.saturating_sub(3);
        let truncated: String = text.chars().take(truncate_to).collect();
        format!("{}...", truncated)
    }
}

/// Wrap text to a specific width
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    textwrap::wrap(text, width)
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Hello, World!", 10), "Hello, ...");
        assert_eq!(truncate("Hi", 10), "Hi");
    }

    #[test]
    fn test_clean_text() {
        let dirty = "  Hello  \n\n  World  \n  ";
        assert_eq!(clean_text(dirty), "Hello\nWorld");
    }
}
