use crate::dependency::dependency_analyzer::DependencyAnalyzer;
use crate::dependency::dependency_reporter::DependencyReporter;
use crate::output::style;
use crate::util::parallel::{log_parallel_status, parse_parallel_flag};
use std::time::Instant;

pub fn execute(path: String, output: Option<String>, no_parallel: bool) {
    let is_parallel = parse_parallel_flag(no_parallel);
    
    let analyzer = DependencyAnalyzer::new()
        .with_parallel(is_parallel);
    let reporter = DependencyReporter::new();
    
    log_parallel_status(is_parallel);
    
    let start_time = Instant::now();
    match analyzer.analyze_dependencies(&path) {
        Ok(graph) => {
            let elapsed = start_time.elapsed();
            reporter.report(&graph);
            style::print_success(&format!("Analysis completed in {:.2?}", elapsed));
            
            if let Some(output_path) = output {
                match reporter.export_dot(&graph, output_path) {
                    Ok(_) => {
                        style::print_success("Dependency graph exported successfully");
                    }
                    Err(e) => {
                        style::print_error(&format!("Error exporting dependency graph: {}", e));
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            style::print_error(&format!("Error analyzing dependencies: {}", e));
            std::process::exit(1);
        }
    }
}