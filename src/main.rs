mod ai;
mod analyzer;
mod cache;
mod dependency;
mod metrics;
mod output;

use ai::AiConfig;

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
    /// Analyze code with AI
    Analyze {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        
        /// Specific file to analyze
        #[arg(short, long)]
        file: Option<String>,
        
        /// Analysis prompt or question
        #[arg(short, long)]
        prompt: Option<String>,
        
        /// AI model tier to use (low, medium, high)
        #[arg(short, long)]
        tier: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load AI configuration (fails silently if no .env file)
    let ai_config = match AiConfig::from_env() {
        Ok(config) => config,
        Err(e) => {
            output::style::print_warning(&format!("AI configuration error: {}. Some features may be limited.", e));
            AiConfig::default()
        }
    };
    
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
        Commands::Analyze { path, file, prompt, tier } => {
            // Parse tier if provided
            let model_tier = if let Some(tier_str) = tier {
                match tier_str.parse() {
                    Ok(t) => Some(t),
                    Err(e) => {
                        output::style::print_error(&format!("Invalid tier: {}. Using default tier.", e));
                        None
                    }
                }
            } else {
                None
            };
            
            // Create AI provider
            let ai_provider = ai::factory::create_ai_provider(ai_config.clone(), model_tier);
            
            output::style::print_info(&format!("Using AI provider: {} ({})", 
                ai_provider.provider_name(), ai_provider.model_name()));
            
            // Handle file-specific or directory analysis
            if let Some(file_path) = file {
                // Read the file
                match std::fs::read_to_string(&file_path) {
                    Ok(content) => {
                        // Analyze the file
                        let analysis_prompt = prompt.as_deref().unwrap_or("Analyze this code and provide insights.");
                        output::style::print_info(&format!("Analyzing file: {}", file_path));
                        
                        match ai_provider.analyze_code(&content, Some(analysis_prompt)).await {
                            Ok(analysis) => {
                                println!("\n{}", analysis);
                            }
                            Err(e) => {
                                output::style::print_error(&format!("AI analysis error: {}", e));
                                process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        output::style::print_error(&format!("Error reading file: {}", e));
                        process::exit(1);
                    }
                }
            } else {
                // Handle directory-level analysis
                let query = prompt.as_deref().unwrap_or("What can you tell me about this codebase?");
                
                output::style::print_info(&format!("Analyzing directory: {}", path));
                output::style::print_info("This may take some time for large codebases...");
                
                // For directory analysis, we would need a more complex implementation
                // Here we just pass a prompt to the AI
                match ai_provider.generate_response(query).await {
                    Ok(response) => {
                        println!("\n{}", response);
                    }
                    Err(e) => {
                        output::style::print_error(&format!("AI analysis error: {}", e));
                        process::exit(1);
                    }
                }
            }
        }
    }
}
