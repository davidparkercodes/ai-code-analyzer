use crate::description::CodeDescriptor;
use crate::output::style;
use crate::output::markdown::render_markdown;
use crate::ai::AiConfig;
use crate::util::parallel::{log_parallel_status, parse_parallel_flag};
use std::time::Instant;

pub async fn execute(path: String, output: Option<String>, no_parallel: bool) {
    let is_parallel = parse_parallel_flag(no_parallel);
    
    let ai_config = match AiConfig::from_env() {
        Ok(config) => config,
        Err(e) => {
            style::print_warning(&format!("AI configuration error: {}. Some features may be limited.", e));
            AiConfig::default()
        }
    };

    let descriptor = CodeDescriptor::new(ai_config)
        .with_parallel(is_parallel);
    
    style::print_header("🤖 Codebase Analysis and Description");
    style::print_info(&format!("📂 Analyzing directory: {}", path));
    
    log_parallel_status(is_parallel);
    
    let start_time = Instant::now();
    match descriptor.describe_codebase(&path).await {
        Ok(description) => {
            let elapsed = start_time.elapsed();
            
            // Format markdown for console and print
            println!("\n{}\n", render_markdown(&description));
            style::print_success(&format!("✨ Description generated in {:.2?}", elapsed));
            
            // Export the description if requested
            if let Some(output_path) = output {
                let path_display = output_path.clone();
                match std::fs::write(output_path, description) {
                    Ok(_) => {
                        style::print_success(&format!("📄 Description exported to {}", path_display));
                    }
                    Err(e) => {
                        style::print_error(&format!("❌ Error writing description: {}", e));
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            style::print_error(&format!("❌ Error generating description: {}", e));
            std::process::exit(1);
        }
    }
}