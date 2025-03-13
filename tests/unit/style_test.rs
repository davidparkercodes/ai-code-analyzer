use code_analyzer::output::style::*;

#[test]
fn test_styled_text_creation() {
    let text = StyledText::new("Test")
        .foreground(Color::Red)
        .background(Color::Blue)
        .style(Style::Bold);

    assert!(text.to_string().contains("Test"));
    assert!(text.to_string().contains("\x1B[")); // Contains ANSI escape code
}

#[test]
fn test_helper_functions() {
    let header_text = header("Header");
    let success_text = success("Success");
    let warning_text = warning("Warning");
    let error_text = error("Error");
    let info_text = info("Info");
    let highlight_text = highlight("Highlight");

    assert!(header_text.to_string().contains("Header"));
    assert!(success_text.to_string().contains("Success"));
    assert!(warning_text.to_string().contains("Warning"));
    assert!(error_text.to_string().contains("Error"));
    assert!(info_text.to_string().contains("Info"));
    assert!(highlight_text.to_string().contains("Highlight"));
}

#[test]
fn test_plain_text() {
    let text = StyledText::new("Plain text");
    assert_eq!(text.to_string(), "Plain text");
}
