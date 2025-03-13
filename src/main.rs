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
    execute(cli).await;
}