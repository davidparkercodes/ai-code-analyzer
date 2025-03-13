mod analyzer;
mod metrics;

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
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { path } => {
            let mut analyzer = analyzer::Analyzer::new();
            if let Err(e) = analyzer.analyze(path) {
                eprintln!("Error analyzing directory: {}", e);
                process::exit(1);
            }
        },
        Commands::Metrics { path } => {
            let collector = metrics::collector::MetricsCollector::new();
            let reporter = metrics::reporter::MetricsReporter::new();
            
            match collector.collect_metrics(path) {
                Ok(metrics) => reporter.report(&metrics),
                Err(e) => {
                    eprintln!("Error analyzing directory: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
