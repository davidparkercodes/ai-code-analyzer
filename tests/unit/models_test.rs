use code_analyzer::metrics::models::{CodeMetrics, LanguageMetrics};

#[test]
fn test_language_metrics_new() {
    let metrics = LanguageMetrics::new("Rust".to_string());

    assert_eq!(metrics.language, "Rust");
    assert_eq!(metrics.files, 0);
    assert_eq!(metrics.lines_of_code, 0);
    assert_eq!(metrics.blank_lines, 0);
    assert_eq!(metrics.comment_lines, 0);
}

#[test]
fn test_code_metrics_new() {
    let metrics = CodeMetrics::new();

    assert_eq!(metrics.total_files, 0);
    assert_eq!(metrics.total_directories, 0);
    assert_eq!(metrics.lines_of_code, 0);
    assert_eq!(metrics.blank_lines, 0);
    assert_eq!(metrics.comment_lines, 0);
    assert!(metrics.by_language.is_empty());
}

#[test]
fn test_add_language_metrics() {
    let mut metrics = CodeMetrics::new();

    let mut lang_metrics = LanguageMetrics::new("Rust".to_string());
    lang_metrics.files = 10;
    lang_metrics.lines_of_code = 500;
    lang_metrics.blank_lines = 100;
    lang_metrics.comment_lines = 200;

    metrics.add_language_metrics(lang_metrics, "/test/file.rs");

    assert_eq!(metrics.lines_of_code, 500);
    assert_eq!(metrics.blank_lines, 100);
    assert_eq!(metrics.comment_lines, 200);
    assert_eq!(metrics.by_language.len(), 1);

    let mut lang_metrics2 = LanguageMetrics::new("Rust".to_string());
    lang_metrics2.files = 5;
    lang_metrics2.lines_of_code = 300;
    lang_metrics2.blank_lines = 50;
    lang_metrics2.comment_lines = 100;

    metrics.add_language_metrics(lang_metrics2, "/test/file2.rs");

    assert_eq!(metrics.lines_of_code, 800);
    assert_eq!(metrics.blank_lines, 150);
    assert_eq!(metrics.comment_lines, 300);
    assert_eq!(metrics.by_language.len(), 1);

    let rust_metrics = metrics.by_language.get("Rust").unwrap();
    assert_eq!(rust_metrics.files, 15);
    assert_eq!(rust_metrics.lines_of_code, 800);

    let mut js_metrics = LanguageMetrics::new("JavaScript".to_string());
    js_metrics.files = 8;
    js_metrics.lines_of_code = 400;
    js_metrics.blank_lines = 80;
    js_metrics.comment_lines = 120;

    metrics.add_language_metrics(js_metrics, "/test/script.js");

    assert_eq!(metrics.lines_of_code, 1200);
    assert_eq!(metrics.blank_lines, 230);
    assert_eq!(metrics.comment_lines, 420);
    assert_eq!(metrics.by_language.len(), 2);
}
