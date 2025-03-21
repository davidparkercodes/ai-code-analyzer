mod run;
mod metrics;
mod dependencies;
mod style;
mod describe;
pub mod delete_comments;
mod clean_code_analyze;
pub mod architecture_diagram;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aicodeanalyzer")]
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
    /// Delete comments from source code files
    #[command(name = "delete-comments")]
    DeleteComments {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        
        /// Programming language to clean comments from (supports 'rust', 'python', 'csharp'/'cs', 'typescript'/'ts')
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
    /// Analyze code against Clean Code principles using AI
    #[command(name = "clean-code-analyze")]
    CleanCodeAnalyze {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        
        /// Custom output path (optional, uses default structured output if not specified)
        #[arg(short, long)]
        output_path: Option<String>,
        
        /// Disable parallel processing for large codebases
        #[arg(long)]
        no_parallel: bool,
        
        /// AI model tier to use (low, medium, high)
        #[arg(long = "ai-level", default_value = "medium")]
        ai_level: String,
        
        /// Focus only on actionable, high-impact recommendations
        #[arg(long)]
        actionable_only: bool,
        
        /// Strictness level for analysis (low: minimal recommendations, medium: standard, high: comprehensive)
        #[arg(long = "analyze-level", default_value = "medium")]
        analyze_level: String,
    },
    /// Generate architecture diagrams from code analysis
    #[command(name = "architecture-diagram")]
    ArchitectureDiagram {
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
        
        /// Diagram format (dot, plantuml, mermaid, c4)
        #[arg(long, default_value = "dot")]
        format: String,
        
        /// Diagram detail level (high, medium, low)
        #[arg(long, default_value = "medium")]
        detail: String,
        
        /// Include test files in the architecture diagram
        #[arg(long)]
        include_tests: bool,
        
        /// Group by module/package instead of individual files
        #[arg(long)]
        group_by_module: bool,
        
        /// Focus on specific module or directory (relative to analysis path)
        #[arg(long)]
        focus: Option<String>,
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
        Commands::DeleteComments { path, language, no_output, output_path, no_parallel, no_git, force, dry_run } => 
            delete_comments::execute(path, language, no_output, output_path, no_parallel, no_git, force, dry_run),
        Commands::CleanCodeAnalyze { path, output_path, no_parallel, ai_level, actionable_only, analyze_level } => 
            clean_code_analyze::execute(path, output_path, no_parallel, ai_level, actionable_only, analyze_level).await,
        Commands::ArchitectureDiagram { path, no_output, output_path, no_parallel, format, detail, include_tests, group_by_module, focus } => 
            architecture_diagram::execute(path, no_output, output_path, no_parallel, format, detail, include_tests, group_by_module, focus).await,
    }
}