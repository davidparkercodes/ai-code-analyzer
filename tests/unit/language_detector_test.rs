use code_analyzer::metrics::language::LanguageDetector;

#[test]
fn test_detect_language() {
    let detector = LanguageDetector::new();
    
    // Test various file extensions
    assert_eq!(detector.detect_language("rs"), "Rust");
    assert_eq!(detector.detect_language("js"), "JavaScript");
    assert_eq!(detector.detect_language("jsx"), "JavaScript");
    assert_eq!(detector.detect_language("ts"), "TypeScript");
    assert_eq!(detector.detect_language("tsx"), "TypeScript");
    assert_eq!(detector.detect_language("py"), "Python");
    assert_eq!(detector.detect_language("unknown"), "Other");
}

#[test]
fn test_get_comment_syntax() {
    let detector = LanguageDetector::new();
    
    // Test comment syntax for different languages
    let (line, block_start, block_end) = detector.get_comment_syntax("Rust");
    assert_eq!(line, "//");
    assert_eq!(block_start, "/*");
    assert_eq!(block_end, "*/");
    
    let (line, block_start, block_end) = detector.get_comment_syntax("Python");
    assert_eq!(line, "#");
    assert_eq!(block_start, "\"\"\"");
    assert_eq!(block_end, "\"\"\"");
    
    // Test unknown language
    let (line, block_start, block_end) = detector.get_comment_syntax("Unknown");
    assert_eq!(line, "");
    assert_eq!(block_start, "");
    assert_eq!(block_end, "");
}