use super::formatter::format_markdown;

/// Format a markdown string for terminal output
pub fn render_markdown(markdown: &str) -> String {
    format_markdown(markdown)
}