mod run;
mod metrics;
mod dependencies;
mod style;
mod describe;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "codeanalyzer")]
#[command(about = "AI-Powered Codebase Analysis Tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
    /// Analyze code style patterns and generate a style guide
    Style {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        
        /// Output path for the style guide markdown file (optional)
        #[arg(short, long)]
        output: Option<String>,
        
        /// Disable parallel processing for large codebases
        #[arg(long)]
        no_parallel: bool,
    },
    /// Generate an AI-powered description of the codebase
    Describe {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        
        /// Output path for the description markdown file (optional)
        #[arg(short, long)]
        output: Option<String>,
        
        /// Disable parallel processing for large codebases
        #[arg(long)]
        no_parallel: bool,
    },
}

pub async fn execute(cli: Cli) {
    match cli.command {
        Commands::Run { path, no_parallel } => run::execute(path, no_parallel),
        Commands::Metrics { path, no_parallel } => metrics::execute(path, no_parallel),
        Commands::Dependencies { path, output, no_parallel } => dependencies::execute(path, output, no_parallel),
        Commands::Style { path, output, no_parallel } => style::execute(path, output, no_parallel),
        Commands::Describe { path, output, no_parallel } => describe::execute(path, output, no_parallel).await,
    }
}