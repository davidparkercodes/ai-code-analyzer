use crate::metrics::collector::MetricsCollector;
use crate::metrics::models::CodeMetrics;
use crate::metrics::reporter::MetricsReporter;
use crate::output::style;
use crate::util::parallel::{log_parallel_status, parse_parallel_flag, ParallelProcessing};

pub fn execute(path: String, no_parallel: bool) -> i32 {
    let parallel_enabled = parse_parallel_flag(no_parallel);
    
    let collector = initialize_metrics_collector(parallel_enabled);
    let reporter = MetricsReporter::new();

    log_parallel_status(parallel_enabled);

    let metrics_result = collect_code_metrics(&collector, &path);
    
    match metrics_result {
        Ok(metrics) => {
            display_metrics_results(&reporter, &metrics);
            0
        },
        Err(error_message) => {
            style::print_error(&error_message);
            1
        }
    }
}

fn initialize_metrics_collector(parallel_enabled: bool) -> MetricsCollector {
    MetricsCollector::new()
        .with_parallel(parallel_enabled)
}

fn collect_code_metrics(
    collector: &MetricsCollector, 
    directory_path: &str
) -> Result<CodeMetrics, String> {
    collector.collect_metrics(directory_path)
        .map_err(|error| format!("Error analyzing directory: {}", error))
}

fn display_metrics_results(
    reporter: &MetricsReporter,
    metrics: &CodeMetrics
) {
    reporter.report(metrics);
    style::print_success("Metrics analysis completed successfully");
}