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
    no_output: bool, 
    output_path: Option<String>, 
    no_parallel: bool,
    ai_level: String
) -> i32 {
    match execute_clean_code_analyze_command(path, no_output, output_path, no_parallel, ai_level).await {
        Ok(_) => 0,
        Err(error) => handle_command_error(&error)
    }
}

async fn execute_clean_code_analyze_command(
    path: String, 
    no_output: bool,
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
    let analysis_result = analyze_clean_code(&analyzer, &path, model, parallel_enabled).await?;
    
    display_analysis_results(&analysis_result, start_time);
    
    if !no_output {
        if let Some(output_path) = custom_output_path {
            export_analysis(&analysis_result, output_path)?;
        } else {
            let default_output = path.clone();
            export_analysis(&analysis_result, default_output)?;
        }
    }
    
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

async fn analyze_clean_code(
    _analyzer: &FileAnalyzer,
    directory_path: &str,
    model: Arc<dyn crate::ai::AiModel>,
    parallel: bool
) -> AppResult<String> {
    let source_files = get_all_source_files(directory_path, parallel)
        .map_err(|e| AppError::Analysis(format!("Error scanning directory: {}", e)))?;
    
    if source_files.is_empty() {
        return Err(AppError::Analysis(format!("No source files found in {}", directory_path)));
    }

    style::print_info(&format!("Found {} source files to analyze", source_files.len()));
    
    let mut all_code = String::new();
    let max_files = 10; // Limit to prevent exceeding AI context limits
    let analyzed_files = source_files.iter().take(max_files).collect::<Vec<_>>();
    
    for file_path in &analyzed_files {
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
        Analyze {} files, noting this is a subset of the codebase:\n{}",
        analyzed_files.len(),
        all_code
    );
    
    style::print_info(&format!("Analyzing code with AI (analyzing {} files)", analyzed_files.len()));
    
    let analysis = model.generate_response(&prompt).await
        .map_err(|e| AppError::Ai(e))?;
    
    Ok(analysis)
}

fn display_analysis_results(analysis: &str, start_time: Instant) {
    let elapsed = start_time.elapsed();
    println!("\n{}\n", render_markdown(analysis));
    style::print_success(&format!("Clean code analysis completed in {:.2?}", elapsed));
}

fn export_analysis(content: &str, file_path: String) -> AppResult<()> {
    let path = crate::output::path::resolve_output_path("clean-code", &file_path, "md")?;
    
    std::fs::write(&path, content)
        .map_err(|error| AppError::FileSystem { 
            path: path.clone(), 
            message: format!("Error writing clean code analysis: {}", error) 
        })?;
    
    style::print_success(&format!("Clean code analysis exported to {}", path.display()));
    Ok(())
}