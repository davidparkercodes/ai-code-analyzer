use ai_code_analyzer::analyzer::Analyzer;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

fn setup_test_directory(dir: &Path) {
    fs::create_dir_all(dir.join("src")).expect("Failed to create subdirectory");

    let rust_content = r#"
fn main() {
    println!("Hello, world!");
}
"#;
    let mut file = File::create(dir.join("src/test.rs")).expect("Failed to create file");
    file.write_all(rust_content.as_bytes())
        .expect("Failed to write to file");
}

#[test]
fn test_analyzer_new() {
    let _analyzer = Analyzer::new();
}

#[test]
fn test_analyzer_analyze() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    setup_test_directory(temp_dir.path());

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(temp_dir.path());

    assert!(result.is_ok());
}

#[test]
fn test_analyzer_analyze_nonexistent_directory() {
    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze("/nonexistent/directory");

    assert!(result.is_err());
}
