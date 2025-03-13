mod ai;
mod analyzer;
mod cache;
mod commands;
mod dependency;
mod description;
mod metrics;
mod output;
mod style_analyzer;
mod util;

use clap::Parser;
use commands::{Cli, execute};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    let exit_code = execute(cli).await;
    
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}