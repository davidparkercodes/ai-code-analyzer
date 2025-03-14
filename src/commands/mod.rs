mod run;
mod metrics;
mod dependencies;
mod style;
mod describe;
pub mod clean_comments;

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
        
        /// Disable auto-saving of the output file
        #[arg(long)]
        no_output: bool,
        
        /// Custom output path (optional, uses default structured output if not specified)
        #[arg(short, long)]
        output_path: Option<String>,
        
        /// Disable parallel processing for large codebases
        #[arg(long)]
        no_parallel: bool,
    },
    /// Analyze dependencies and generate a dependency graph
    Dependencies {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        
        /// Disable auto-saving of the output file
        #[arg(long)]
        no_output: bool,
        
        /// Custom output path (optional, uses default structured output if not specified)
        #[arg(short, long)]
        output_path: Option<String>,
        
        /// Disable parallel processing for large codebases
        #[arg(long)]
        no_parallel: bool,
    },
    /// Analyze code style patterns and generate a style guide
    Style {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        
        /// Disable auto-saving of the output file
        #[arg(long)]
        no_output: bool,
        
        /// Custom output path (optional, uses default structured output if not specified)
        #[arg(short, long)]
        output_path: Option<String>,
        
        /// Disable parallel processing for large codebases
        #[arg(long)]
        no_parallel: bool,
    },
    /// Generate an AI-powered description of the codebase
    Describe {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        
        /// Disable auto-saving of the output file
        #[arg(long)]
        no_output: bool,
        
        /// Custom output path (optional, uses default structured output if not specified)
        #[arg(short, long)]
        output_path: Option<String>,
        
        /// Disable parallel processing for large codebases
        #[arg(long)]
        no_parallel: bool,
    },
    /// Clean comments from source code files
    #[command(name = "clean-comments")]
    CleanComments {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        
        /// Programming language to clean comments from (currently only 'rust' is supported)
        #[arg(short, long, required = true)]
        language: String,
        
        /// Disable auto-saving of cleaned files to output directory
        #[arg(long)]
        no_output: bool,
        
        /// Custom output directory for cleaned files (optional, uses default structured output if not specified)
        #[arg(short, long)]
        output_path: Option<String>,
        
        /// Disable parallel processing for large codebases
        #[arg(long)]
        no_parallel: bool,
        
        /// Skip Git operations (checking for repo, adding files, committing)
        #[arg(long)]
        no_git: bool,
        
        /// Force removing comments without asking for confirmation in non-git directories
        #[arg(long)]
        force: bool,
        
        /// Show what would be removed without making changes
        #[arg(long)]
        dry_run: bool,
    },
}

pub async fn execute(cli: Cli) -> i32 {
    match cli.command {
        Commands::Run { path, no_parallel } => run::execute(path, no_parallel),
        Commands::Metrics { path, no_output, output_path, no_parallel } => 
            metrics::execute(path, no_output, output_path, no_parallel),
        Commands::Dependencies { path, no_output, output_path, no_parallel } => 
            dependencies::execute(path, no_output, output_path, no_parallel),
        Commands::Style { path, no_output, output_path, no_parallel } => 
            style::execute(path, no_output, output_path, no_parallel),
        Commands::Describe { path, no_output, output_path, no_parallel } => 
            describe::execute(path, no_output, output_path, no_parallel).await,
        Commands::CleanComments { path, language, no_output, output_path, no_parallel, no_git, force, dry_run } => 
            clean_comments::execute(path, language, no_output, output_path, no_parallel, no_git, force, dry_run),
    }
}