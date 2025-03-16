use crate::metrics::collector::MetricsCollector;
use crate::metrics::models::CodeMetrics;
use crate::metrics::reporter::MetricsReporter;
use crate::output::style;
use crate::util::error::{AppError, AppResult, handle_command_error};
use crate::util::parallel::{ParallelProcessing, log_parallel_status, parse_parallel_flag};

pub fn execute(
    path: String,
    no_output: bool,
    output_path: Option<String>,
    no_parallel: bool,
) -> i32 {
    match execute_metrics_command(path, no_output, output_path, no_parallel) {
        Ok(_) => 0,
        Err(error) => handle_command_error(&error),
    }
}

fn execute_metrics_command(
    path: String,
    no_output: bool,
    custom_output_path: Option<String>,
    no_parallel: bool,
) -> AppResult<()> {
    let parallel_enabled = parse_parallel_flag(no_parallel);

    let collector = initialize_metrics_collector(parallel_enabled);
    let reporter = MetricsReporter::new();

    log_parallel_status(parallel_enabled);

    let metrics = collect_code_metrics(&collector, &path)?;
    display_metrics_results(&reporter, &metrics);

    if !no_output {
        if let Some(output_path) = custom_output_path {
            export_metrics(&reporter, &metrics, output_path)?;
        } else {
            let default_output = path.clone();
            export_metrics(&reporter, &metrics, default_output)?;
        }
    }

    Ok(())
}

// This should be deleted
fn initialize_metrics_collector(parallel_enabled: bool) -> MetricsCollector {
    MetricsCollector::new().enable_parallel_processing(parallel_enabled)
}

fn collect_code_metrics(
    collector: &MetricsCollector,
    directory_path: &str,
) -> AppResult<CodeMetrics> {
    collector
        .collect_metrics(directory_path)
        .map_err(|error| AppError::Metrics(format!("Error analyzing directory: {}", error)))
}

fn display_metrics_results(reporter: &MetricsReporter, metrics: &CodeMetrics) {
    reporter.report(metrics);
    style::print_success("Metrics analysis completed successfully");
}

fn export_metrics(
    reporter: &MetricsReporter,
    metrics: &CodeMetrics,
    output_path: String,
) -> AppResult<()> {
    let path = crate::output::path::resolve_output_path("metrics", &output_path, "md")?;

    reporter
        .export_metrics(metrics, &path)
        .map_err(|error| AppError::FileSystem {
            path: path.clone(),
            message: format!("Error exporting metrics: {}", error),
        })?;

    style::print_success(&format!("Metrics exported to {}", path.display()));
    Ok(())
}
