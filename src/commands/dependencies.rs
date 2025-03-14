use crate::dependency::dependency_analyzer::DependencyAnalyzer;
use crate::dependency::dependency_graph::DependencyGraph;
use crate::dependency::dependency_reporter::DependencyReporter;
use crate::output::style;
use crate::util::error::{AppError, AppResult, handle_command_error};
use crate::util::parallel::{log_parallel_status, parse_parallel_flag, ParallelProcessing};
use std::time::Instant;

pub fn execute(path: String, no_output: bool, output_path: Option<String>, no_parallel: bool) -> i32 {
    match execute_dependencies_command(path, no_output, output_path, no_parallel) {
        Ok(_) => 0,
        Err(error) => handle_command_error(&error)
    }
}

fn execute_dependencies_command(
    path: String, 
    no_output: bool, 
    custom_output_path: Option<String>,
    no_parallel: bool
) -> AppResult<()> {
    let parallel_enabled = parse_parallel_flag(no_parallel);
    
    let analyzer = initialize_analyzer(parallel_enabled);
    let reporter = DependencyReporter::new();
    
    log_parallel_status(parallel_enabled);
    
    let start_time = Instant::now();
    let graph = perform_dependency_analysis(&analyzer, &path)?;
    
    display_analysis_results(&reporter, &graph, start_time);
    
    if !no_output {
        if let Some(output_path) = custom_output_path {
            export_dependency_graph(&reporter, &graph, output_path)?;
        } else {
            let default_output = path.clone();
            export_dependency_graph(&reporter, &graph, default_output)?;
        }
    }
    
    Ok(())
}

fn initialize_analyzer(parallel_enabled: bool) -> DependencyAnalyzer {
    DependencyAnalyzer::new()
        .enable_parallel_processing(parallel_enabled)
}

fn perform_dependency_analysis(
    analyzer: &DependencyAnalyzer, 
    directory_path: &str
) -> AppResult<DependencyGraph> {
    analyzer.analyze_dependencies(directory_path)
        .map_err(|error| AppError::Dependency(format!("Error analyzing dependencies: {}", error)))
}

fn display_analysis_results(
    reporter: &DependencyReporter,
    graph: &DependencyGraph,
    start_time: Instant
) {
    let elapsed = start_time.elapsed();
    reporter.report(graph);
    style::print_success(&format!("Analysis completed in {:.2?}", elapsed));
}

fn export_dependency_graph(
    reporter: &DependencyReporter,
    graph: &DependencyGraph,
    output_path: String
) -> AppResult<()> {
    reporter.export_dot(graph, output_path)
        .map_err(|error| AppError::Dependency(format!("Error exporting dependency graph: {}", error)))?;
    
    style::print_success("Dependency graph exported successfully");
    Ok(())
}
