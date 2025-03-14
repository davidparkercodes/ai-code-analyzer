use crate::output::style;
use crate::output::markdown::render_markdown;
use crate::ai::{AiConfig, ModelTier, factory};
use crate::util::error::{AppError, AppResult, handle_command_error};
use crate::util::parallel::{log_parallel_status, parse_parallel_flag};
use crate::util::file_filter::get_all_source_files;
use std::time::{Instant, Duration};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Configuration struct for the Clean Code Analyze command
struct CleanCodeConfig {
    path: String,
    output_path: String,
    parallel_enabled: bool,
    model_tier: ModelTier,
    only_recommendations: bool,
}

/// Analysis configuration for a single batch
struct BatchAnalysisConfig<'a> {
    batch: &'a FileBatch<'a>,
    model: Arc<dyn crate::ai::AiModel>,
    only_recommendations: bool,
}

/// Result of a batch analysis
struct BatchAnalysisResult {
    content: String,
    batch_number: usize,
    file_count: usize,
    elapsed: Duration,
}

/// File batch information
struct FileBatch<'a> {
    files: &'a [PathBuf],
    batch_number: usize,
    batch_count: usize,
}

pub async fn execute(
    path: String, 
    output_path: Option<String>, 
    no_parallel: bool,
    ai_level: String,
    only_recommendations: bool
) -> i32 {
    match execute_clean_code_analysis(path, output_path, no_parallel, ai_level, only_recommendations).await {
        Ok(_) => 0,
        Err(error) => handle_command_error(&error)
    }
}

async fn execute_clean_code_analysis(
    path: String, 
    custom_output_path: Option<String>, 
    no_parallel: bool,
    ai_level: String,
    only_recommendations: bool
) -> AppResult<()> {
    // Parse configuration
    let config = prepare_command_config(
        path, 
        custom_output_path.unwrap_or_default(), 
        no_parallel, 
        &ai_level, 
        only_recommendations
    )?;
    
    // Initialize AI model
    let model = initialize_ai_model(&config.model_tier)?;
    
    // Scan for source files
    let source_files = scan_source_files(&config.path, config.parallel_enabled)?;
    
    // Process files in batches
    analyze_code_in_batches(&config, &source_files, model).await
}

fn prepare_command_config(
    path: String,
    custom_output_path: String,
    no_parallel: bool,
    ai_level: &str,
    only_recommendations: bool
) -> AppResult<CleanCodeConfig> {
    let parallel_enabled = parse_parallel_flag(no_parallel);
    let model_tier = parse_model_tier(ai_level)?;
    let output_path = if custom_output_path.is_empty() { path.clone() } else { custom_output_path };
    
    display_analysis_header(&path);
    log_parallel_status(parallel_enabled);
    
    Ok(CleanCodeConfig {
        path,
        output_path,
        parallel_enabled,
        model_tier,
        only_recommendations,
    })
}

fn initialize_ai_model(tier: &ModelTier) -> AppResult<Arc<dyn crate::ai::AiModel>> {
    let ai_config = load_ai_configuration()?;
    factory::create_ai_model(ai_config, tier.clone())
        .map_err(|e| AppError::Ai(e))
}

fn scan_source_files(path: &str, parallel_enabled: bool) -> AppResult<Vec<PathBuf>> {
    let start_time = Instant::now();
    
    let source_files = get_all_source_files(path, parallel_enabled)
        .map_err(|e| AppError::Analysis(format!("Error scanning directory: {}", e)))?;
    
    if source_files.is_empty() {
        return Err(AppError::Analysis(format!("No source files found in {}", path)));
    }

    style::print_info(&format!("📂 Found {} source files to analyze in {:.2?}", 
        source_files.len(), start_time.elapsed()));
    
    Ok(source_files)
}

async fn analyze_code_in_batches(
    config: &CleanCodeConfig,
    source_files: &[PathBuf],
    model: Arc<dyn crate::ai::AiModel>
) -> AppResult<()> {
    let start_time = Instant::now();
    
    // Create batches
    let batches = create_file_batches(source_files);
    log_batch_processing_start(&batches);
    
    // Process each batch
    process_all_batches(&batches, model, config).await?;
    
    // Log completion
    log_processing_complete(start_time.elapsed());
    Ok(())
}

fn log_batch_processing_start(batches: &[FileBatch]) {
    let max_batch_size = batches.first().map_or(0, |b| b.files.len());
    style::print_info(&format!("🔄 Processing {} batches of up to {} files each", 
        batches.len(), max_batch_size));
}

fn log_processing_complete(elapsed: Duration) {
    style::print_success(&format!("✅ All batches processed in {:.2?}", elapsed));
}

async fn process_all_batches(
    batches: &[FileBatch<'_>],
    model: Arc<dyn crate::ai::AiModel>,
    config: &CleanCodeConfig
) -> AppResult<()> {
    for batch in batches {
        // Create batch analysis config
        let batch_config = BatchAnalysisConfig {
            batch,
            model: model.clone(),
            only_recommendations: config.only_recommendations,
        };
        
        // Analyze the batch
        let batch_result = analyze_code_batch(&batch_config).await?;
        
        // Display and export results
        process_batch_results(&batch_result, config)?;
    }
    
    Ok(())
}

fn process_batch_results(
    result: &BatchAnalysisResult,
    config: &CleanCodeConfig
) -> AppResult<()> {
    // Display results
    display_batch_results(result);
    
    // Export to file
    export_batch_analysis(
        &result.content, 
        &config.output_path, 
        result.batch_number, 
        &config.model_tier
    )
}

fn create_file_batches(source_files: &[PathBuf]) -> Vec<FileBatch> {
    let batch_size = 10;
    let batch_count = (source_files.len() + batch_size - 1) / batch_size; // Ceiling division
    
    (0..batch_count)
        .map(|batch_index| {
            let start_idx = batch_index * batch_size;
            let end_idx = std::cmp::min(start_idx + batch_size, source_files.len());
            let batch_files = &source_files[start_idx..end_idx];
            
            FileBatch {
                files: batch_files,
                batch_number: batch_index + 1,
                batch_count,
            }
        })
        .collect()
}

async fn analyze_code_batch(config: &BatchAnalysisConfig<'_>) -> AppResult<BatchAnalysisResult> {
    let batch = config.batch;
    
    style::print_info(&format!("⏳ Analyzing batch {}/{} ({} files)", 
        batch.batch_number, batch.batch_count, batch.files.len()));
    
    let start_time = Instant::now();
    
    // Collect file contents
    let file_contents = collect_file_contents(batch.files)?;
    
    // Create AI prompt
    let prompt = create_ai_prompt(
        &file_contents, 
        batch.batch_number, 
        batch.files.len(), 
        config.only_recommendations
    );
    
    style::print_info(&format!("🧠 Analyzing code with AI (batch #{}: {} files)", 
        batch.batch_number, batch.files.len()));
    
    // Generate AI analysis
    let analysis = config.model.generate_response(&prompt).await
        .map_err(|e| AppError::Ai(e))?;
    
    Ok(BatchAnalysisResult {
        content: analysis,
        batch_number: batch.batch_number,
        file_count: batch.files.len(),
        elapsed: start_time.elapsed(),
    })
}

fn collect_file_contents(files: &[PathBuf]) -> AppResult<Vec<(String, String)>> {
    files.iter().map(|file_path| {
        let display_path = file_path.display().to_string();
        let content = fs::read_to_string(file_path)
            .map_err(|e| AppError::Analysis(
                format!("Error reading file {}: {}", display_path, e)
            ))?;
        
        Ok((display_path, content))
    }).collect()
}

fn create_ai_prompt(
    file_contents: &[(String, String)], 
    batch_number: usize, 
    file_count: usize,
    only_recommendations: bool
) -> String {
    // Concatenate file contents
    let all_code = file_contents.iter()
        .map(|(path, content)| format!("\n\n// File: {}\n{}", path, content))
        .collect::<Vec<_>>()
        .join("");
    
    if only_recommendations {
        create_recommendations_prompt(all_code, batch_number, file_count)
    } else {
        create_full_analysis_prompt(all_code, batch_number, file_count)
    }
}

fn create_shared_prompt_base() -> String {
    "Analyze the following code against these Clean Code principles:\n\
    - Use meaningful and intention-revealing names\n\
    - Functions should do one thing only and do it well\n\
    - Keep functions small (preferably under 20 lines)\n\
    - Arguments should be few (ideally 0-2, maximum 3)\n\
    - Avoid side effects in functions\n\
    - Don't repeat yourself (DRY)\n\
    - Maintain clear separation of concerns\n\
    - Avoid unnecessary comments (code should be self-documenting)".to_string()
}

fn create_recommendations_prompt(code: String, batch_number: usize, file_count: usize) -> String {
    let base_prompt = create_shared_prompt_base();
    
    format!(
        "{}\n\n\
        For each principle, ONLY identify violations and problematic code. Then provide actionable \
        recommendations on how to improve the code to better follow Clean Code principles. \
        Be constructive and specific. Include line numbers or function names in your recommendations \
        whenever possible.\n\n\
        IMPORTANT: If the code already follows good practices for a principle, you can state that \
        no violations were found for that principle. It's perfectly acceptable to say the code looks \
        good in some or all areas if that is the case.\n\n\
        Analyze these {} files (Batch #{}):\n{}",
        base_prompt,
        file_count,
        batch_number,
        code
    )
}

fn create_full_analysis_prompt(code: String, batch_number: usize, file_count: usize) -> String {
    let base_prompt = create_shared_prompt_base();
    
    format!(
        "{}\n\n\
        For each principle, indicate whether the code follows it, with specific examples of good practices \
        or violations found. Then provide actionable recommendations on how to improve the code to better \
        follow Clean Code principles. Be constructive and specific. Include line numbers or function names \
        in your recommendations whenever possible.\n\n\
        IMPORTANT: If the code already follows good practices, you don't need to force recommendations. \
        You can simply acknowledge that the code is well-structured and follows Clean Code principles \
        for those areas.\n\n\
        Analyze these {} files (Batch #{}):\n{}",
        base_prompt,
        file_count,
        batch_number,
        code
    )
}

fn display_analysis_header(directory_path: &str) {
    style::print_header("🔍 Analyzing Clean Code Principles");
    style::print_info(&format!("📂 Analyzing directory: {}", directory_path));
}

fn display_batch_results(result: &BatchAnalysisResult) {
    println!("\n{}\n", render_markdown(&result.content));
    style::print_success(&format!(
        "✨ Clean code analysis for batch #{} ({} files) completed in {:.2?}", 
        result.batch_number, result.file_count, result.elapsed
    ));
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

fn parse_model_tier(level: &str) -> AppResult<ModelTier> {
    level.parse::<ModelTier>().map_err(|e| {
        AppError::Analysis(format!("Invalid AI level: {}. Use 'low', 'medium', or 'high'", e))
    })
}

fn export_batch_analysis(
    content: &str, 
    base_path: &str, 
    batch_number: usize, 
    model_tier: &ModelTier
) -> AppResult<()> {
    use chrono::Local;
    
    // Get the directory name from base_path
    let dir_name = Path::new(base_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .replace(".", "_");
    
    // Create filename with directory name, batch number, AI level, and timestamp
    let timestamp = Local::now().timestamp();
    let output_name = "clean-code-analyze";
    let file_name = format!(
        "{}_batch{}_{}_{}",
        dir_name, 
        batch_number, 
        format!("{:?}", model_tier).to_lowercase(), 
        timestamp
    );
    
    let path = crate::output::path::resolve_output_path(output_name, &file_name, "md")?;
    
    std::fs::write(&path, content)
        .map_err(|error| AppError::FileSystem { 
            path: path.clone(), 
            message: format!("Error writing clean code analysis: {}", error) 
        })?;
    
    style::print_success(&format!(
        "📄 Clean code analysis batch #{} exported to {}", 
        batch_number, 
        path.display()
    ));
    
    Ok(())
}