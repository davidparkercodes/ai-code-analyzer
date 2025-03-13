use code_analyzer::metrics::models::{CodeMetrics, LanguageMetrics};
use code_analyzer::metrics::reporter::MetricsReporter;

#[test]
fn test_reporter_new() {
    // Just ensure we can create a new reporter instance
    let reporter = MetricsReporter::new();
    reporter.report(&CodeMetrics::new());
}

#[test]
fn test_reporter_report() {
    let mut metrics = CodeMetrics::new();
    metrics.total_files = 10;
    metrics.total_directories = 5;
    metrics.lines_of_code = 500;
    metrics.blank_lines = 100;
    metrics.comment_lines = 200;

    // Add language-specific metrics
    let mut rust_metrics = LanguageMetrics::new("Rust".to_string());
    rust_metrics.files = 5;
    rust_metrics.lines_of_code = 300;
    rust_metrics.blank_lines = 60;
    rust_metrics.comment_lines = 120;
    metrics.add_language_metrics(rust_metrics, "/src/main.rs");

    let mut js_metrics = LanguageMetrics::new("JavaScript".to_string());
    js_metrics.files = 3;
    js_metrics.lines_of_code = 150;
    js_metrics.blank_lines = 30;
    js_metrics.comment_lines = 60;
    metrics.add_language_metrics(js_metrics, "/src/app.js");

    let reporter = MetricsReporter::new();

    // This just tests that report runs without error
    // Since it only prints to stdout, we can't easily assert the output
    reporter.report(&metrics);
}
