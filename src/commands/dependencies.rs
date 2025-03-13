use crate::dependency::dependency_analyzer::DependencyAnalyzer;
use crate::dependency::dependency_graph::DependencyGraph;
use crate::dependency::dependency_reporter::DependencyReporter;
use crate::output::style;
use crate::util::parallel::{log_parallel_status, parse_parallel_flag, ParallelProcessing};
use std::time::Instant;

pub fn execute(path: String, output: Option<String>, no_parallel: bool) -> i32 {
    let parallel_enabled = parse_parallel_flag(no_parallel);
    
    let analyzer = initialize_analyzer(parallel_enabled);
    let reporter = DependencyReporter::new();
    
    log_parallel_status(parallel_enabled);
    
    let start_time = Instant::now();
    let analysis_result = perform_dependency_analysis(&analyzer, &path);
    
    match analysis_result {
        Ok(graph) => {
            display_analysis_results(&reporter, &graph, start_time);
            
            if let Some(output_path) = output {
                if let Err(exit_code) = export_dependency_graph(&reporter, &graph, output_path) {
                    return exit_code;
                }
            }
            0
        }
        Err(error_message) => {
            style::print_error(&error_message);
            1
        }
    }
}

fn initialize_analyzer(parallel_enabled: bool) -> DependencyAnalyzer {
    DependencyAnalyzer::new()
        .with_parallel(parallel_enabled)
}

fn perform_dependency_analysis(
    analyzer: &DependencyAnalyzer, 
    directory_path: &str
) -> Result<DependencyGraph, String> {
    analyzer.analyze_dependencies(directory_path)
        .map_err(|error| format!("Error analyzing dependencies: {}", error))
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
) -> Result<(), i32> {
    match reporter.export_dot(graph, output_path) {
        Ok(_) => {
            style::print_success("Dependency graph exported successfully");
            Ok(())
        }
        Err(error) => {
            style::print_error(&format!("Error exporting dependency graph: {}", error));
            Err(1)
        }
    }
}