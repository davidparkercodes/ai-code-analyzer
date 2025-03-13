mod analyzer;
mod cache;
mod dependency;
mod metrics;
mod output;

use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "codeanalyzer")]
#[command(about = "AI-Powered Codebase Analysis Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the code analyzer on the specified directory
    Run {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        
        /// Disable parallel processing for large codebases
        #[arg(long)]
        no_parallel: bool,
    },
    /// Generate only code metrics for the specified directory
    Metrics {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        
        /// Disable parallel processing for large codebases
        #[arg(long)]
        no_parallel: bool,
    },
    /// Analyze dependencies and generate a dependency graph
    Dependencies {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        
        /// Output path for the DOT graph file (optional)
        #[arg(short, long)]
        output: Option<String>,
        
        /// Disable parallel processing for large codebases
        #[arg(long)]
        no_parallel: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { path, no_parallel } => {
            let mut analyzer = analyzer::Analyzer::new()
                .with_parallel(!no_parallel);
            if let Err(e) = analyzer.analyze(path) {
                output::style::print_error(&format!("Error analyzing directory: {}", e));
                process::exit(1);
            }
        }
        Commands::Metrics { path, no_parallel } => {
            let collector = metrics::collector::MetricsCollector::new()
                .with_parallel(!no_parallel);
            let reporter = metrics::reporter::MetricsReporter::new();

            match collector.collect_metrics(path) {
                Ok(metrics) => {
                    reporter.report(&metrics);
                    if !no_parallel {
                        output::style::print_success("Parallel processing: enabled");
                    }
                },
                Err(e) => {
                    output::style::print_error(&format!("Error analyzing directory: {}", e));
                    process::exit(1);
                }
            }
        }
        Commands::Dependencies { path, output, no_parallel } => {
            let analyzer = dependency::dependency_analyzer::DependencyAnalyzer::new()
                .with_parallel(!no_parallel);
            let reporter = dependency::dependency_reporter::DependencyReporter::new();
            
            if !no_parallel {
                output::style::print_info("Parallel processing: enabled");
            }
            
            let start_time = std::time::Instant::now();
            match analyzer.analyze_dependencies(path) {
                Ok(graph) => {
                    let elapsed = start_time.elapsed();
                    reporter.report(&graph);
                    output::style::print_success(&format!("Analysis completed in {:.2?}", elapsed));
                    
                    if let Some(output_path) = output {
                        match reporter.export_dot(&graph, output_path) {
                            Ok(_) => {
                                output::style::print_success("Dependency graph exported successfully");
                            }
                            Err(e) => {
                                output::style::print_error(&format!("Error exporting dependency graph: {}", e));
                                process::exit(1);
                            }
                        }
                    }
                }
                Err(e) => {
                    output::style::print_error(&format!("Error analyzing dependencies: {}", e));
                    process::exit(1);
                }
            }
        }
    }
}
