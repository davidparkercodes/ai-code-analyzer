mod analyzer;
mod dependency;
mod metrics;
mod output;
mod style;

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
    },
    /// Generate only code metrics for the specified directory
    Metrics {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Analyze dependencies and generate a dependency graph
    Dependencies {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        
        /// Output path for the DOT graph file (optional)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Analyze code style and generate style recommendations
    Style {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { path } => {
            let mut analyzer = analyzer::Analyzer::new();
            if let Err(e) = analyzer.analyze(path) {
                output::style::print_error(&format!("Error analyzing directory: {}", e));
                process::exit(1);
            }
        }
        Commands::Metrics { path } => {
            let collector = metrics::collector::MetricsCollector::new();
            let reporter = metrics::reporter::MetricsReporter::new();

            match collector.collect_metrics(path) {
                Ok(metrics) => reporter.report(&metrics),
                Err(e) => {
                    output::style::print_error(&format!("Error analyzing directory: {}", e));
                    process::exit(1);
                }
            }
        }
        Commands::Dependencies { path, output } => {
            let analyzer = dependency::dependency_analyzer::DependencyAnalyzer::new();
            let reporter = dependency::dependency_reporter::DependencyReporter::new();
            
            match analyzer.analyze_dependencies(path) {
                Ok(graph) => {
                    reporter.report(&graph);
                    
                    if let Some(output_path) = output {
                        match reporter.export_dot(&graph, output_path) {
                            Ok(_) => {
                                output::style::print_success(&format!("Dependency graph exported successfully"));
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
        Commands::Style { path } => {
            let detector = style::detector::StyleDetector::new();
            let reporter = style::reporter::StyleReporter::new();
            
            // Using current directory if path is not specified
            let target_path = if path.is_empty() { "." } else { &path };
            
            output::style::print_info(&format!("Analyzing code style in directory: {}", target_path));
            
            match detector.detect_styles(target_path) {
                Ok(analysis) => {
                    reporter.report(&analysis);
                }
                Err(e) => {
                    output::style::print_error(&format!("Error analyzing code style: {}", e));
                    process::exit(1);
                }
            }
        }
    }
}
