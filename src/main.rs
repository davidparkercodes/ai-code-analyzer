use clap::{Parser, Subcommand};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "codeanalyzer")]
#[command(about = "AI-Powered Codebase Analysis Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the code analyzer on the current directory
    Run {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { path } => {
            println!("Analyzing directory: {}", path);
            list_files_and_directories(path);
        }
    }
}

fn list_files_and_directories(dir_path: &str) {
    let path = Path::new(dir_path);
    
    if !path.exists() {
        println!("Error: Path '{}' does not exist", dir_path);
        return;
    }

    println!("\nFiles and directories:");
    println!("----------------------");
    
    for entry in WalkDir::new(path).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.to_string_lossy() == dir_path {
            continue; // Skip the root directory itself
        }
        
        let file_type = if path.is_dir() { "Directory" } else { "File" };
        println!("{}: {}", file_type, path.display());
    }
}
