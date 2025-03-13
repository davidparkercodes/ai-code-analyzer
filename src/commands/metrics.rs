use crate::metrics::collector::MetricsCollector;
use crate::metrics::reporter::MetricsReporter;
use crate::output::style;
use crate::util::parallel::{log_parallel_status, parse_parallel_flag};

pub fn execute(path: String, no_parallel: bool) {
    let is_parallel = parse_parallel_flag(no_parallel);
    
    let collector = MetricsCollector::new()
        .with_parallel(is_parallel);
    let reporter = MetricsReporter::new();

    log_parallel_status(is_parallel);

    match collector.collect_metrics(&path) {
        Ok(metrics) => {
            reporter.report(&metrics);
            style::print_success(&format!("Metrics analysis completed successfully"));
        },
        Err(e) => {
            style::print_error(&format!("Error analyzing directory: {}", e));
            std::process::exit(1);
        }
    }
}