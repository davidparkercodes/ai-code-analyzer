use code_analyzer::style_analyzer::{StyleAnalyzer, StylePattern, StyleRule};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_style_analyzer_integration() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();
    
    // Create test files with different styles
    create_test_file(dir_path, "test1.rs", "fn main() {\n    let x = 5;\n    println!(\"x = {}\", x);\n}\n");
    create_test_file(dir_path, "test2.rs", "fn another_function() {\n    let someValue = 10;\n    println!(\"Value: {}\", someValue);\n}\n");
    
    // Run style analyzer
    let analyzer = StyleAnalyzer::new();
    let result = analyzer.analyze_codebase(dir_path);
    
    assert!(result.is_ok());
    let report = result.unwrap();
    
    // Check that we have patterns detected
    assert!(!report.get_patterns().is_empty());
    
    // Verify style guide generation
    assert!(report.get_style_guide().is_some());
}

#[test]
fn test_style_pattern_creation() {
    use code_analyzer::style_analyzer::pattern::{IndentationStyle, NamingConvention};
    
    // Create and test indentation pattern
    let rule = StyleRule::IndentationStyle(IndentationStyle::Spaces(4));
    let mut pattern = StylePattern::new(rule, "rust");
    
    pattern.add_occurrence(Some("fn test() {\n    let x = 5;\n}".to_string()));
    pattern.update_consistency(10);
    
    assert_eq!(pattern.occurrences, 1);
    assert_eq!(pattern.consistency, 0.1);
    assert_eq!(pattern.language, "rust");
    assert_eq!(pattern.examples.len(), 1);
    
    // Create and test naming convention pattern
    let rule = StyleRule::NamingConvention(NamingConvention::CamelCase);
    let mut pattern = StylePattern::new(rule, "javascript");
    
    pattern.add_occurrence(Some("let userName = 'John';".to_string()));
    pattern.add_occurrence(Some("function calculateTotal() { }".to_string()));
    pattern.update_consistency(5);
    
    assert_eq!(pattern.occurrences, 2);
    assert_eq!(pattern.consistency, 0.4);
    assert_eq!(pattern.language, "javascript");
    assert_eq!(pattern.examples.len(), 2);
}

// Helper function to create test files
fn create_test_file<P: AsRef<Path>>(dir: P, filename: &str, content: &str) {
    let file_path = dir.as_ref().join(filename);
    fs::write(file_path, content).unwrap();
}