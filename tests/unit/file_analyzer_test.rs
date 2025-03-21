use ai_code_analyzer::analyzer::file_analyzer::FileAnalyzer;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

fn create_test_file(dir: &Path, filename: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.join(filename);
    let mut file = File::create(&file_path).expect("Failed to create test file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to test file");
    file_path
}

#[test]
fn test_analyze_file_rust() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let rust_content = r#"
fn main() {
    println!("Hello, world!");
    
    /* Block comment
     * spanning multiple lines
     */
    let x = 42;
}
"#;

    let file_path = create_test_file(temp_dir.path(), "test.rs", rust_content);

    let analyzer = FileAnalyzer::new();
    let metrics = analyzer
        .analyze_file(file_path)
        .expect("Failed to analyze file");

    println!(
        "Rust file metrics: loc={}, blank={}, comments={}",
        metrics.lines_of_code, metrics.blank_lines, metrics.comment_lines
    );

    assert_eq!(metrics.language, "Rust");
    assert_eq!(metrics.files, 1);
    assert_eq!(metrics.lines_of_code, 4);
    assert_eq!(metrics.blank_lines, 2);
    assert_eq!(metrics.comment_lines, 3);
}

#[test]
fn test_analyze_file_python() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let python_content = r#"
# This is a comment
def main():
    # Another comment
    print("Hello, world!")
    
    """
    This is a docstring
    spanning multiple lines
    """
    x = 42
"#;

    let file_path = create_test_file(temp_dir.path(), "test.py", python_content);

    let analyzer = FileAnalyzer::new();
    let metrics = analyzer
        .analyze_file(file_path)
        .expect("Failed to analyze file");

    println!(
        "Python file metrics: loc={}, blank={}, comments={}",
        metrics.lines_of_code, metrics.blank_lines, metrics.comment_lines
    );

    assert_eq!(metrics.language, "Python");
    assert_eq!(metrics.files, 1);
    assert_eq!(metrics.lines_of_code, 7);
    assert_eq!(metrics.blank_lines, 2);
    assert_eq!(metrics.comment_lines, 2);
}

#[test]
fn test_analyze_nonexistent_file() {
    let analyzer = FileAnalyzer::new();
    let metrics = analyzer.analyze_file("nonexistent_file.rs");

    assert!(metrics.is_none());
}
