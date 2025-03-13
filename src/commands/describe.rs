use crate::description::CodeDescriptor;
use crate::output::style;
use crate::output::markdown::render_markdown;
use crate::ai::AiConfig;
use crate::util::error::{AppError, AppResult, handle_command_error};
use crate::util::parallel::{log_parallel_status, parse_parallel_flag, ParallelProcessing};
use std::time::Instant;
use std::path::Path;

pub async fn execute(path: String, output: Option<String>, no_parallel: bool) -> i32 {
    match execute_describe_command(path, output, no_parallel).await {
        Ok(_) => 0,  // Success exit code
        Err(error) => handle_command_error(&error)
    }
}

async fn execute_describe_command(
    path: String, 
    output: Option<String>, 
    no_parallel: bool
) -> AppResult<()> {
    let parallel_enabled = parse_parallel_flag(no_parallel);
    
    let ai_config = load_ai_configuration()?;
    let descriptor = initialize_code_descriptor(ai_config, parallel_enabled);
    
    display_analysis_header(&path);
    log_parallel_status(parallel_enabled);
    
    let start_time = Instant::now();
    let description = generate_codebase_description(&descriptor, &path).await?;
    
    display_description_results(&description, start_time);
    
    if let Some(output_path) = output {
        export_description(&description, output_path)?;
    }
    
    Ok(())
}

fn load_ai_configuration() -> AppResult<AiConfig> {
    match AiConfig::from_env() {
        Ok(config) => Ok(config),
        Err(error) => {
            style::print_warning(&format!(
                "AI configuration error: {}. Using default configuration.", 
                error
            ));
            Ok(AiConfig::default())
        }
    }
}

fn initialize_code_descriptor(ai_config: AiConfig, parallel_enabled: bool) -> CodeDescriptor {
    CodeDescriptor::new(ai_config)
        .with_parallel(parallel_enabled)
}

fn display_analysis_header(directory_path: &str) {
    style::print_header("🤖 Codebase Analysis and Description");
    style::print_info(&format!("📂 Analyzing directory: {}", directory_path));
}

async fn generate_codebase_description(
    descriptor: &CodeDescriptor,
    directory_path: &str
) -> AppResult<String> {
    descriptor.describe_codebase(directory_path).await
        .map_err(|error| AppError::Description(format!("❌ Error generating description: {}", error)))
}

fn display_description_results(description: &str, start_time: Instant) {
    let elapsed = start_time.elapsed();
    println!("\n{}\n", render_markdown(description));
    style::print_success(&format!("✨ Description generated in {:.2?}", elapsed));
}

fn export_description(content: &str, file_path: String) -> AppResult<()> {
    let path = Path::new(&file_path);
    std::fs::write(path, content)
        .map_err(|error| AppError::FileSystem { 
            path: path.to_path_buf(), 
            message: format!("Error writing description: {}", error) 
        })?;
    
    style::print_success(&format!("📄 Description exported to {}", file_path));
    Ok(())
}