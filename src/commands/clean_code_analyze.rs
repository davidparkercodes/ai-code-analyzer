use crate::analyzer::file_analyzer::FileAnalyzer;
use crate::output::style;
use crate::output::markdown::render_markdown;
use crate::ai::{AiConfig, ModelTier, factory};
use crate::util::error::{AppError, AppResult, handle_command_error};
use crate::util::parallel::{log_parallel_status, parse_parallel_flag};
use crate::util::file_filter::get_all_source_files;
use std::time::Instant;
use std::fs;
use std::sync::Arc;

pub async fn execute(
    path: String, 
    output_path: Option<String>, 
    no_parallel: bool,
    ai_level: String
) -> i32 {
    match execute_clean_code_analyze_command(path, output_path, no_parallel, ai_level).await {
        Ok(_) => 0,
        Err(error) => handle_command_error(&error)
    }
}

async fn execute_clean_code_analyze_command(
    path: String, 
    custom_output_path: Option<String>, 
    no_parallel: bool,
    ai_level: String
) -> AppResult<()> {
    let parallel_enabled = parse_parallel_flag(no_parallel);
    let analyzer = FileAnalyzer::new();
    
    display_analysis_header(&path);
    log_parallel_status(parallel_enabled);
    
    let tier = parse_ai_level(&ai_level)?;
    let ai_config = load_ai_configuration()?;
    let model = factory::create_ai_model(ai_config, tier)?;
    
    let start_time = Instant::now();
    let source_files = get_all_source_files(&path, parallel_enabled)
        .map_err(|e| AppError::Analysis(format!("Error scanning directory: {}", e)))?;
    
    if source_files.is_empty() {
        return Err(AppError::Analysis(format!("No source files found in {}", path)));
    }

    style::print_info(&format!("Found {} source files to analyze", source_files.len()));
    
    // Process files in batches of 10
    let batch_size = 10;
    let batch_count = (source_files.len() + batch_size - 1) / batch_size; // Ceiling division
    
    style::print_info(&format!("Processing {} batches of up to {} files each", batch_count, batch_size));
    
    for batch_index in 0..batch_count {
        let start_idx = batch_index * batch_size;
        let end_idx = std::cmp::min(start_idx + batch_size, source_files.len());
        let batch_files = &source_files[start_idx..end_idx];
        
        style::print_info(&format!("Processing batch {}/{} ({} files)", 
            batch_index + 1, batch_count, batch_files.len()));
        
        let analysis_result = analyze_clean_code_batch(
            &analyzer, batch_files, model.clone(), batch_index + 1
        ).await?;
        
        display_analysis_results(&analysis_result, start_time.elapsed());
        
        // Always output to file
        let output_path = if let Some(ref custom_path) = custom_output_path {
            custom_path.clone()
        } else {
            path.clone()
        };
        
        export_analysis(&analysis_result, &output_path, batch_index + 1)?;
    }
    
    let elapsed = start_time.elapsed();
    style::print_success(&format!("All batches processed in {:.2?}", elapsed));
    
    Ok(())
}

fn display_analysis_header(directory_path: &str) {
    style::print_header("Analyzing Clean Code Principles");
    style::print_info(&format!("Analyzing directory: {}", directory_path));
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

fn parse_ai_level(level: &str) -> AppResult<ModelTier> {
    level.parse::<ModelTier>().map_err(|e| {
        AppError::Analysis(format!("Invalid AI level: {}. Use 'low', 'medium', or 'high'", e))
    })
}

async fn analyze_clean_code_batch(
    _analyzer: &FileAnalyzer,
    batch_files: &[std::path::PathBuf],
    model: Arc<dyn crate::ai::AiModel>,
    batch_number: usize
) -> AppResult<String> {
    let mut all_code = String::new();
    
    for file_path in batch_files {
        let file_content = fs::read_to_string(file_path)
            .map_err(|e| AppError::Analysis(format!("Error reading file {}: {}", file_path.display(), e)))?;
        
        all_code.push_str(&format!("\n\n// File: {}\n{}", file_path.display(), file_content));
    }
    
    let prompt = format!(
        "Analyze the following code and evaluate if it follows Clean Code principles:\n\
        - Use meaningful and intention-revealing names\n\
        - Functions should do one thing only and do it well\n\
        - Keep functions small (preferably under 20 lines)\n\
        - Arguments should be few (ideally 0-2, maximum 3)\n\
        - Avoid side effects in functions\n\
        - Don't repeat yourself (DRY)\n\
        - Maintain clear separation of concerns\n\n\
        For each principle, indicate whether the code follows it, with specific examples of good practices \
        or violations found. Then provide actionable recommendations on how to improve the code to better \
        follow Clean Code principles. Be constructive and specific. Include line numbers or function names \
        in your recommendations whenever possible.\n\n\
        Analyze these {} files (Batch #{}):\n{}",
        batch_files.len(),
        batch_number,
        all_code
    );
    
    style::print_info(&format!("Analyzing code with AI (batch #{}: {} files)", 
        batch_number, batch_files.len()));
    
    let analysis = model.generate_response(&prompt).await
        .map_err(|e| AppError::Ai(e))?;
    
    Ok(analysis)
}

fn display_analysis_results(analysis: &str, elapsed: std::time::Duration) {
    println!("\n{}\n", render_markdown(analysis));
    style::print_success(&format!("Clean code analysis completed in {:.2?}", elapsed));
}

fn export_analysis(content: &str, file_path: &str, batch_number: usize) -> AppResult<()> {
    // Include the batch number in the output path
    let output_name = format!("clean-code-batch{}", batch_number);
    let path = crate::output::path::resolve_output_path(&output_name, file_path, "md")?;
    
    std::fs::write(&path, content)
        .map_err(|error| AppError::FileSystem { 
            path: path.clone(), 
            message: format!("Error writing clean code analysis: {}", error) 
        })?;
    
    style::print_success(&format!("Clean code analysis batch #{} exported to {}", batch_number, path.display()));
    Ok(())
}