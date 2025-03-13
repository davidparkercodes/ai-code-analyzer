use code_analyzer::metrics::collector::MetricsCollector;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

fn setup_test_directory(dir: &Path) {
    // Create subdirectories
    fs::create_dir_all(dir.join("src/utils")).expect("Failed to create subdirectory");
    fs::create_dir_all(dir.join("tests")).expect("Failed to create subdirectory");

    // Create Rust file
    let rust_content = r#"
// main.rs
fn main() {
    println!("Hello, world!");
    
    let x = 42;
}
"#;
    let mut file = File::create(dir.join("src/main.rs")).expect("Failed to create file");
    file.write_all(rust_content.as_bytes())
        .expect("Failed to write to file");

    // Create Python file
    let py_content = r#"
# script.py
def main():
    print("Hello, world!")
    
    x = 42
"#;
    let mut file = File::create(dir.join("src/utils/script.py")).expect("Failed to create file");
    file.write_all(py_content.as_bytes())
        .expect("Failed to write to file");

    // Create JavaScript file
    let js_content = r#"
// app.js
function main() {
    console.log("Hello, world!");
    
    let x = 42;
}
"#;
    let mut file = File::create(dir.join("src/app.js")).expect("Failed to create file");
    file.write_all(js_content.as_bytes())
        .expect("Failed to write to file");
}

#[test]
fn test_collect_metrics() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    setup_test_directory(temp_dir.path());

    let collector = MetricsCollector::new();
    let result = collector.collect_metrics(temp_dir.path());

    assert!(result.is_ok());

    let metrics = result.unwrap();

    // We should have 3 files and 3 directories (root, src, src/utils, tests)
    assert_eq!(metrics.total_files, 3);
    assert_eq!(metrics.total_directories, 4); // temp_dir, src, src/utils, tests

    // Check language breakdown
    assert_eq!(metrics.by_language.len(), 3); // Rust, Python, JavaScript

    // Verify Rust metrics
    let rust_metrics = metrics
        .by_language
        .get("Rust")
        .expect("No Rust metrics found");
    assert_eq!(rust_metrics.files, 1);

    // Verify Python metrics
    let py_metrics = metrics
        .by_language
        .get("Python")
        .expect("No Python metrics found");
    assert_eq!(py_metrics.files, 1);

    // Verify JavaScript metrics
    let js_metrics = metrics
        .by_language
        .get("JavaScript")
        .expect("No JavaScript metrics found");
    assert_eq!(js_metrics.files, 1);
}

#[test]
fn test_collect_metrics_nonexistent_directory() {
    let collector = MetricsCollector::new();
    let result = collector.collect_metrics("/nonexistent/directory");

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_collect_metrics_file_as_input() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.txt");

    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(b"test content")
        .expect("Failed to write to file");

    let collector = MetricsCollector::new();
    let result = collector.collect_metrics(&file_path);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("is not a directory"));
}
