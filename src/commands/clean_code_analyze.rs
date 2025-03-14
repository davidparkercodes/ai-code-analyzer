use crate::output::style;
use crate::ai::{AiConfig, ModelTier, factory};
use crate::ai::prompts::clean_code_analyze as prompt;
use crate::util::error::{AppError, AppResult, handle_command_error};
use crate::util::parallel::{log_parallel_status, parse_parallel_flag};
use crate::util::file_filter::get_all_source_files;
use std::time::{Instant, Duration};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use serde_json;

/// Representation of an actionable recommendation
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActionableItem {
    location: String,
    recommendation: String,
}

/// JSON representation of a clean code analysis file result
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileAnalysisResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    _ordering_field1: Option<()>,
    file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    _ordering_field2: Option<()>,
    score: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    _ordering_field3: Option<()>,
    actionable_items: Vec<ActionableItem>,
}

/// Collection of file analysis results in JSON format
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchAnalysisJson {
    batch_number: usize,
    results: Vec<FileAnalysisResult>,
}

/// Custom struct used only for serializing JSON in the correct order
#[derive(Debug, Serialize)]
struct OrderedAnalysisResult {
    file: String,
    score: u32,
    #[serde(rename = "actionableItems")]
    actionable_items: Vec<OrderedActionableItem>,
}

/// Custom struct for ordered actionable items
#[derive(Debug, Serialize)]
struct OrderedActionableItem {
    location: String,
    recommendation: String,
}

/// Strictness level for code analysis
#[derive(Debug, Clone)]
enum AnalyzeLevel {
    /// Minimal recommendations - only the most critical issues
    Low,
    /// Standard analysis with balanced approach
    Medium,
    /// Comprehensive analysis with detailed recommendations
    High,
}

impl FromStr for AnalyzeLevel {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(AnalyzeLevel::Low),
            "medium" => Ok(AnalyzeLevel::Medium),
            "high" => Ok(AnalyzeLevel::High),
            _ => Err(format!("Invalid analyze level: {}. Use 'low', 'medium', or 'high'", s))
        }
    }
}

impl std::fmt::Display for AnalyzeLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalyzeLevel::Low => write!(f, "low"),
            AnalyzeLevel::Medium => write!(f, "medium"),
            AnalyzeLevel::High => write!(f, "high"),
        }
    }
}

/// Configuration struct for the Clean Code Analyze command
struct CleanCodeConfig {
    path: String,
    output_path: String,
    parallel_enabled: bool,
    model_tier: ModelTier,
    actionable_only: bool,
    analyze_level: AnalyzeLevel,
}

/// Analysis configuration for a single batch
struct BatchAnalysisConfig<'a> {
    batch: &'a FileBatch<'a>,
    model: Arc<dyn crate::ai::AiModel>,
    actionable_only: bool,
    analyze_level: AnalyzeLevel,
}

/// Result of a batch analysis
struct BatchAnalysisResult {
    content: String,
    batch_number: usize,
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
    actionable_only: bool,
    analyze_level: String
) -> i32 {
    match execute_clean_code_analysis(path, output_path, no_parallel, ai_level, actionable_only, analyze_level).await {
        Ok(_) => 0,
        Err(error) => handle_command_error(&error)
    }
}

async fn execute_clean_code_analysis(
    path: String, 
    custom_output_path: Option<String>, 
    no_parallel: bool,
    ai_level: String,
    actionable_only: bool,
    analyze_level_str: String
) -> AppResult<()> {
    let config = prepare_command_config(
        path, 
        custom_output_path.unwrap_or_default(), 
        no_parallel, 
        &ai_level, 
        actionable_only,
        &analyze_level_str
    )?;
    
    let model = initialize_ai_model(&config.model_tier)?;
    
    let source_files = scan_source_files(&config.path, config.parallel_enabled)?;
    
    analyze_code_in_batches(&config, &source_files, model).await
}

fn prepare_command_config(
    path: String,
    custom_output_path: String,
    no_parallel: bool,
    ai_level: &str,
    actionable_only: bool,
    analyze_level_str: &str
) -> AppResult<CleanCodeConfig> {
    let parallel_enabled = parse_parallel_flag(no_parallel);
    let model_tier = parse_model_tier(ai_level)?;
    let analyze_level = parse_analyze_level(analyze_level_str)?;
    let output_path = if custom_output_path.is_empty() { path.clone() } else { custom_output_path };
    
    display_analysis_header(&path);
    log_parallel_status(parallel_enabled);
    log_analyze_level(&analyze_level);
    style::print_info("📊 Output format: JSON");
    
    Ok(CleanCodeConfig {
        path,
        output_path,
        parallel_enabled,
        model_tier,
        actionable_only,
        analyze_level,
    })
}

fn initialize_ai_model(tier: &ModelTier) -> AppResult<Arc<dyn crate::ai::AiModel>> {
    let ai_config = load_ai_configuration()?;
    factory::create_ai_model(ai_config, tier.clone())
        .map_err(|e| AppError::Ai(e))
}

fn scan_source_files(path: &str, parallel_enabled: bool) -> AppResult<Vec<PathBuf>> {
    let start_time = Instant::now();
    
    let source_files = get_source_files(path, parallel_enabled)?;
    
    validate_source_files(&source_files, path)?;

    log_scan_results(&source_files, start_time.elapsed());
    
    Ok(source_files)
}

fn get_source_files(path: &str, parallel_enabled: bool) -> AppResult<Vec<PathBuf>> {
    let all_files = get_all_source_files(path, parallel_enabled)
        .map_err(|e| map_scan_error(e))?;
    
    // Filter out known binary and cache files at scan time
    let text_files: Vec<PathBuf> = all_files.into_iter()
        .filter(|path| !should_skip_file(path))
        .collect();
    
    Ok(text_files)
}

fn map_scan_error(error: std::io::Error) -> AppError {
    AppError::Analysis(format!("Error scanning directory: {}", error))
}

fn validate_source_files(files: &[PathBuf], path: &str) -> AppResult<()> {
    if files.is_empty() {
        return Err(AppError::Analysis(format!("No source files found in {}", path)));
    }
    Ok(())
}

fn log_scan_results(files: &[PathBuf], elapsed: Duration) {
    style::print_info(&format!("📂 Found {} source files to analyze in {:.2?}", 
        files.len(), elapsed));
}

async fn analyze_code_in_batches(
    config: &CleanCodeConfig,
    source_files: &[PathBuf],
    model: Arc<dyn crate::ai::AiModel>
) -> AppResult<()> {
    let start_time = Instant::now();
    
    let batches = create_file_batches(source_files);
    log_batch_processing_start(&batches);
    
    process_all_batches(&batches, model, config).await?;
    
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
    let mut processed_batches = 0;
    
    for batch in batches {
        let batch_config = BatchAnalysisConfig {
            batch,
            model: model.clone(),
            actionable_only: config.actionable_only,
            analyze_level: config.analyze_level.clone(),
        };
        
        if let Some(batch_result) = analyze_code_batch(&batch_config).await? {
            process_batch_results(&batch_result, config)?;
            processed_batches += 1;
        }
    }
    
    if processed_batches == 0 {
        style::print_warning("No batches could be processed - no valid text files found");
    }
    
    Ok(())
}

fn process_batch_results(
    result: &BatchAnalysisResult,
    config: &CleanCodeConfig
) -> AppResult<()> {
    let res = export_batch_analysis(
        &result.content, 
        &config.output_path, 
        result.batch_number, 
        &config.model_tier,
        config.actionable_only,
        &config.analyze_level
    );
    
    if res.is_ok() {
        style::print_info(&format!("✅ Batch #{} analysis complete", result.batch_number));
    }
    
    res
}


fn create_file_batches(source_files: &[PathBuf]) -> Vec<FileBatch> {
    let batch_size = 10;
    let batch_count = (source_files.len() + batch_size - 1) / batch_size;
    
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

async fn analyze_code_batch(config: &BatchAnalysisConfig<'_>) -> AppResult<Option<BatchAnalysisResult>> {
    let batch = config.batch;
    
    style::print_info(&format!("⏳ Analyzing batch {}/{} ({} files)", 
        batch.batch_number, batch.batch_count, batch.files.len()));
    
    let start_time = Instant::now();
    
    let file_contents = collect_file_contents(batch.files)?;
    let valid_file_count = file_contents.len();
    
    // Skip batch if no valid files
    if valid_file_count == 0 {
        style::print_warning(&format!("Skipping batch #{} - no valid files to analyze", batch.batch_number));
        return Ok(None);
    }
    
    style::print_info(&format!("🔄 Processing {} valid files in batch #{}", valid_file_count, batch.batch_number));
    
    let prompt = create_ai_prompt(
        &file_contents, 
        batch.batch_number, 
        valid_file_count, 
        config.actionable_only,
        &config.analyze_level
    );
    
    style::print_info(&format!("🧠 Analyzing code with AI (batch #{}: {} files)", 
        batch.batch_number, valid_file_count));
    
    let analysis = config.model.generate_response(&prompt).await
        .map_err(|e| AppError::Ai(e))?;
    
    let elapsed = start_time.elapsed();
    style::print_info(&format!("⌛ AI analysis completed in {:.2?}", elapsed));
    
    Ok(Some(BatchAnalysisResult {
        content: analysis,
        batch_number: batch.batch_number,
    }))
}

fn collect_file_contents(files: &[PathBuf]) -> AppResult<Vec<(String, String)>> {
    let mut valid_files = Vec::new();
    
    for file_path in files.iter() {
        // Skip binary files and cache directories by default
        if should_skip_file(file_path) {
            style::print_info(&format!("Skipping binary/cache file: {}", file_path.display()));
            continue;
        }
        
        match read_file_with_path(file_path) {
            Ok(file_content) => valid_files.push(file_content),
            Err(error) => {
                // Log the error but continue with other files
                style::print_warning(&format!("Skipping file: {}", error));
            }
        }
    }
    
    // Return an empty collection instead of an error if no valid files
    // This allows the batch to be skipped gracefully
    Ok(valid_files)
}

fn should_skip_file(file_path: &PathBuf) -> bool {
    let path_str = file_path.to_string_lossy();
    
    // Skip __pycache__ directory files
    if path_str.contains("__pycache__") {
        return true;
    }
    
    // Skip common binary file extensions
    let is_binary = [
        ".pyc", ".exe", ".dll", ".so", ".dylib", 
        ".bin", ".dat", ".jpg", ".jpeg", ".png", 
        ".gif", ".class", ".jar", ".o", ".a"
    ].iter().any(|ext| path_str.ends_with(ext));
    
    is_binary
}

fn read_file_with_path(file_path: &PathBuf) -> Result<(String, String), String> {
    let display_path = file_path.display().to_string();
    
    match fs::read_to_string(file_path) {
        Ok(content) => Ok((display_path, content)),
        Err(e) => Err(format!("Error reading file {}: {}", display_path, e))
    }
}

fn create_ai_prompt(
    file_contents: &[(String, String)], 
    batch_number: usize, 
    file_count: usize,
    actionable_only: bool,
    analyze_level: &AnalyzeLevel
) -> String {
    prompt::create_clean_code_json_prompt(
        file_contents,
        batch_number,
        file_count,
        actionable_only,
        analyze_level.to_string().as_str()
    )
}

fn display_analysis_header(directory_path: &str) {
    style::print_header("🔍 Analyzing Clean Code Principles");
    style::print_info(&format!("📂 Analyzing directory: {}", directory_path));
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

fn parse_analyze_level(level: &str) -> AppResult<AnalyzeLevel> {
    level.parse::<AnalyzeLevel>().map_err(|e| {
        AppError::Analysis(format!("Invalid analyze level: {}. Use 'low', 'medium', or 'high'", e))
    })
}

fn log_analyze_level(level: &AnalyzeLevel) {
    let description = match level {
        AnalyzeLevel::Low => "minimal (only critical issues)",
        AnalyzeLevel::Medium => "standard (balanced approach)",
        AnalyzeLevel::High => "comprehensive (detailed analysis)"
    };
    
    style::print_info(&format!("🔍 Analysis strictness: {} - {}", level, description));
}

fn export_batch_analysis(
    content: &str, 
    base_path: &str, 
    batch_number: usize, 
    model_tier: &ModelTier,
    actionable_only: bool,
    analyze_level: &AnalyzeLevel
) -> AppResult<()> {
    match serde_json::from_str::<serde_json::Value>(content) {
        Ok(json_value) => {
            let path = generate_output_path(
                base_path, 
                batch_number, 
                model_tier, 
                actionable_only, 
                analyze_level
            )?;
            
            let file_count = if let serde_json::Value::Array(ref array) = json_value {
                array.len()
            } else {
                0
            };
            
            let ordered_json = if let serde_json::Value::Array(array) = json_value {
                let ordered_array = array.iter().map(|item| {
                    if let serde_json::Value::Object(map) = item {
                        let mut ordered_map = serde_json::Map::new();
                        
                        ordered_map.insert("file".to_string(), map.get("file")
                            .cloned()
                            .unwrap_or(serde_json::Value::String("unknown".to_string())));
                            
                        ordered_map.insert("score".to_string(), map.get("score")
                            .cloned()
                            .unwrap_or(serde_json::Value::Number(serde_json::Number::from(0))));
                            
                        if let Some(items) = map.get("actionableItems") {
                            if let serde_json::Value::Array(items_array) = items {
                                let ordered_items = items_array.iter().map(|item| {
                                    if let serde_json::Value::Object(item_map) = item {
                                        let mut ordered_item = serde_json::Map::new();
                                        
                                        ordered_item.insert("location".to_string(), item_map.get("location")
                                            .cloned()
                                            .unwrap_or(serde_json::Value::String("unknown".to_string())));
                                            
                                        ordered_item.insert("recommendation".to_string(), item_map.get("recommendation")
                                            .cloned()
                                            .unwrap_or(serde_json::Value::String("".to_string())));
                                        
                                        serde_json::Value::Object(ordered_item)
                                    } else {
                                        item.clone()
                                    }
                                }).collect();
                                
                                ordered_map.insert("actionableItems".to_string(), serde_json::Value::Array(ordered_items));
                            } else {
                                ordered_map.insert("actionableItems".to_string(), items.clone());
                            }
                        }
                        
                        serde_json::Value::Object(ordered_map)
                    } else {
                        item.clone()
                    }
                }).collect();
                
                serde_json::Value::Array(ordered_array)
            } else {
                json_value
            };
            
            let ordered_results: Vec<OrderedAnalysisResult> = if let serde_json::Value::Array(array) = &ordered_json {
                array.iter().filter_map(|item| {
                    if let serde_json::Value::Object(map) = item {
                        let file = map.get("file")?.as_str()?.to_string();
                        let score = map.get("score")?.as_u64()? as u32;
                        
                        let actionable_items = if let Some(serde_json::Value::Array(items)) = map.get("actionableItems") {
                            items.iter().filter_map(|item| {
                                if let serde_json::Value::Object(item_map) = item {
                                    let location = item_map.get("location")?.as_str()?.to_string();
                                    let recommendation = item_map.get("recommendation")?.as_str()?.to_string();
                                    
                                    Some(OrderedActionableItem {
                                        location,
                                        recommendation,
                                    })
                                } else {
                                    None
                                }
                            }).collect()
                        } else {
                            Vec::new()
                        };
                        
                        Some(OrderedAnalysisResult {
                            file,
                            score,
                            actionable_items,
                        })
                    } else {
                        None
                    }
                }).collect()
            } else {
                Vec::new()
            };
            
            let formatted_content = serde_json::to_string_pretty(&ordered_results)
                .unwrap_or_else(|_| content.to_string());
            
            write_analysis_to_file(&path, &formatted_content)?;
            
            log_export_success(batch_number, file_count, &path);
            
            Ok(())
        },
        Err(e) => {
            style::print_warning(&format!("Invalid JSON response: {}", e));
            Err(AppError::Analysis(format!("Failed to parse JSON response: {}", e)))
        }
    }
}

fn generate_output_path(
    base_path: &str, 
    batch_number: usize, 
    model_tier: &ModelTier,
    actionable_only: bool,
    analyze_level: &AnalyzeLevel
) -> AppResult<std::path::PathBuf> {
    use chrono::Local;
    
    let dir_name = Path::new(base_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .replace(".", "_");
    
    let timestamp = Local::now().timestamp();
    let output_name = "clean-code-analyze";
    
    let model_tier_str = format!("level-{}", format!("{:?}", model_tier).to_lowercase());
    let analyze_level_str = format!("analyze-{}", analyze_level);
    
    let file_name = if actionable_only {
        format!(
            "{}_batch{}_{}_{}_{}_{}", 
            dir_name, 
            batch_number, 
            model_tier_str,
            analyze_level_str,
            "actionable-only", 
            timestamp
        )
    } else {
        format!(
            "{}_batch{}_{}_{}_{}", 
            dir_name, 
            batch_number, 
            model_tier_str,
            analyze_level_str,
            timestamp
        )
    };
    
    crate::output::path::resolve_output_path(output_name, &file_name, "json")
}

fn write_analysis_to_file(path: &std::path::Path, content: &str) -> AppResult<()> {
    std::fs::write(path, content)
        .map_err(|error| AppError::FileSystem { 
            path: path.to_path_buf(), 
            message: format!("Error writing clean code analysis: {}", error) 
        })
}

fn log_export_success(batch_number: usize, file_count: usize, path: &std::path::Path) {
    style::print_success(&format!(
        "📄 Clean code JSON analysis for batch #{} ({} files) exported to {}", 
        batch_number, 
        file_count,
        path.display()
    ));
}
