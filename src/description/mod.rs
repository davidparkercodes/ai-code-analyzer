use std::path::Path;
use std::sync::{Arc, Mutex};
use std::fs;
use std::collections::HashMap;

use ignore::{DirEntry, WalkBuilder};
use rayon::prelude::*;

use crate::ai::{AiConfig, ModelTier, factory, AiModel, AiError};
use crate::cache::AnalysisCache;
use crate::metrics::language::LanguageDetector;
use crate::output::style;
use crate::util::error::{AppError, AppResult};
use crate::util::file_filter::FileFilter;
use crate::util::parallel::ParallelProcessing;

const BATCH_SIZE: usize = 10;

/// A structure to hold file content and metadata for AI analysis
#[derive(Debug, Clone)]
pub struct FileData {
    pub path: String,
    pub content: String,
    pub language: String,
}

/// A structure representing a batch of files to analyze
#[derive(Debug, Clone)]
pub struct FileBatch {
    pub files: Vec<FileData>,
    pub base_path: String,
}

/// Main structure for describing codebases
pub struct CodeDescriptor {
    language_detector: LanguageDetector,
    cache: Arc<AnalysisCache>,
    ai_config: AiConfig,
    parallel: bool,
}

impl ParallelProcessing for CodeDescriptor {
    fn enable_parallel_processing(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }
    
    fn with_parallel(self, parallel: bool) -> Self {
        self.enable_parallel_processing(parallel)
    }
    
    fn is_parallel(&self) -> bool {
        self.parallel
    }
}

impl CodeDescriptor {
    /// Create a new CodeDescriptor
    pub fn new(ai_config: AiConfig) -> Self {
        CodeDescriptor {
            language_detector: LanguageDetector::new(),
            cache: Arc::new(AnalysisCache::new()),
            ai_config,
            parallel: true,
        }
    }
    
    /// Describe a codebase using AI
    pub async fn describe_codebase<P: AsRef<Path>>(&self, dir_path: P) -> AppResult<String> {
        let path = dir_path.as_ref();
        
        if !path.exists() {
            return Err(AppError::Description(format!("Path '{}' does not exist", path.display())));
        }

        if !path.is_dir() {
            return Err(AppError::Description(format!("Path '{}' is not a directory", path.display())));
        }
        
        style::print_info("🔎 Scanning codebase files...");
        
        // Collect and process files
        let batches = self.build_file_batches(path)?;
        style::print_info(&format!("📦 Collected {} file batches for analysis", batches.len()));
        
        // Generate batch summaries using low-tier model
        style::print_info("🚀 Starting AI analysis process...");
        let batch_summaries = self.generate_batch_summaries(&batches).await?;
        style::print_info(&format!("📝 Generated {} batch summaries", batch_summaries.len()));
        
        // Generate final description using high-tier model
        let description = self.generate_final_description(&batch_summaries).await?;
        
        style::print_info("🎉 Codebase analysis complete!");
        
        Ok(description)
    }
    
    /// Exposed for file collection operation
    #[allow(dead_code)]
    pub fn collect_files<P: AsRef<Path>>(&self, dir_path: P) -> AppResult<Vec<FileBatch>> {
        self.build_file_batches(dir_path)
    }

    // Main file collection implementation
    fn build_file_batches<P: AsRef<Path>>(&self, dir_path: P) -> AppResult<Vec<FileBatch>> {
        let path = dir_path.as_ref();
        
        // Create walker that respects .gitignore
        let walker = WalkBuilder::new(path)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .filter_entry(|e| {
                !FileFilter::should_exclude(e.path())
            })
            .build();
        
        // Collect all entries first
        let entries: Vec<DirEntry> = walker
            .filter_map(|result| {
                match result {
                    Ok(entry) => Some(entry),
                    Err(err) => {
                        // Proper error logging but continue without failing
                        style::print_warning(&format!("Warning during file scan: {}", err));
                        None
                    }
                }
            })
            .collect();
        
        // Filter to get only file entries
        let file_entries: Vec<&DirEntry> = entries
            .iter()
            .filter(|e| e.path().is_file())
            .collect();
        
        style::print_info(&format!("Found {} files for analysis", file_entries.len()));
        
        // Process file entries
        let file_data = Arc::new(Mutex::new(Vec::<FileData>::new()));
        
        let process_entry = |entry: &&DirEntry| {
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            
            // Skip binary files and very large files
            if let Ok(metadata) = fs::metadata(path) {
                if metadata.len() > 500000 { // Skip files larger than ~500KB
                    return;
                }
            }
            
            // Get file content
            let content = if let Some(cached_content) = self.cache.get_file_content(&path_str) {
                cached_content
            } else if let Ok(file_content) = fs::read_to_string(path) {
                self.cache.cache_file_content(&path_str, file_content.clone());
                file_content
            } else {
                return; // Skip if we can't read the file
            };
            
            // Skip empty files
            if content.trim().is_empty() {
                return;
            }
            
            // Detect language
            let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            
            let language = if let Some(cached_lang) = self.cache.get_language(&path_str) {
                cached_lang
            } else {
                let detected_lang = if extension.is_empty() && !file_name.is_empty() {
                    self.language_detector.detect_by_filename(file_name)
                } else {
                    self.language_detector.detect_language(extension)
                };
                self.cache.cache_language(&path_str, detected_lang.clone());
                detected_lang
            };
            
            // Add the file data
            let mut file_data_guard = file_data.lock().unwrap();
            file_data_guard.push(FileData {
                path: path_str,
                content,
                language,
            });
        };
        
        // Process files in parallel or sequentially
        if self.parallel {
            file_entries.par_iter().for_each(process_entry);
        } else {
            file_entries.iter().for_each(process_entry);
        }
        
        // Create file batches
        let all_files = file_data.lock().unwrap().clone();
        
        // Group files by directory structure
        let mut grouped_files: HashMap<String, Vec<FileData>> = HashMap::new();
        
        for file in all_files {
            // Get the directory path by removing the file name
            let path = Path::new(&file.path);
            let parent = path.parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| String::from(""));
            
            grouped_files.entry(parent.clone())
                .or_insert_with(Vec::new)
                .push(file);
        }
        
        // Create batches
        let mut batches = Vec::new();
        
        for (dir_path, files) in grouped_files {
            // If the directory has more files than the batch size, split them
            if files.len() > BATCH_SIZE {
                for chunk in files.chunks(BATCH_SIZE) {
                    batches.push(FileBatch {
                        files: chunk.to_vec(),
                        base_path: dir_path.clone(),
                    });
                }
            } else {
                batches.push(FileBatch {
                    files,
                    base_path: dir_path,
                });
            }
        }
        
        Ok(batches)
    }

    // Generate summaries for each batch of files using the low-tier AI model
    async fn generate_batch_summaries(&self, batches: &[FileBatch]) -> AppResult<Vec<String>> {
        let mut summaries = Vec::new();
        
        // Create the low-tier AI model
        let low_tier_model = self.create_low_tier_model()?;
        
        style::print_info("Generating batch summaries with AI...");
        style::print_info(&format!("Processing {} batch(es) with low-tier model", batches.len()));
        
        // We'll process each batch sequentially, regardless of the parallel flag
        // This is to avoid async runtime issues
        for (batch_index, batch) in batches.iter().enumerate() {
            let batch_desc = self.format_batch_description(batch);
            let file_count = batch.files.len();
            
            // Log batch beginning
            self.log_batch_processing_start(batch_index, batches.len(), file_count, &batch_desc);
            
            // Format files and create prompt
            let file_texts = self.prepare_files_for_analysis(&batch.files);
            let prompt = self.create_batch_analysis_prompt(&batch.base_path, &file_texts);
            
            // Call the AI model and process result
            self.process_batch_result(
                &low_tier_model, 
                &prompt, 
                &mut summaries, 
                batch_index, 
                batches.len(), 
                file_count, 
                &batch_desc
            ).await;
        }
        
        Ok(summaries)
    }
    
    // Create a low-tier AI model for batch processing
    fn create_low_tier_model(&self) -> AppResult<Box<dyn AiModel>> {
        factory::create_ai_model(self.ai_config.clone(), ModelTier::Low)
            .map_err(|e| AppError::Ai(e))
    }
    
    // Format batch description for logging
    fn format_batch_description(&self, batch: &FileBatch) -> String {
        if batch.base_path.is_empty() {
            "root directory".to_string()
        } else {
            batch.base_path.clone()
        }
    }
    
    // Log the start of batch processing
    fn log_batch_processing_start(&self, batch_index: usize, total_batches: usize, file_count: usize, batch_desc: &str) {
        style::print_info(&format!(
            "🔍 [Batch {}/{}] Processing {} files from {}", 
            batch_index + 1, 
            total_batches, 
            file_count,
            batch_desc
        ));
    }
    
    // Format all files in a batch for AI analysis
    fn prepare_files_for_analysis(&self, files: &[FileData]) -> Vec<String> {
        let mut file_texts = Vec::new();
        
        for file in files {
            let formatted_file = self.format_file_for_analysis(file);
            file_texts.push(formatted_file);
        }
        
        file_texts
    }
    
    // Format a single file for AI analysis
    fn format_file_for_analysis(&self, file: &FileData) -> String {
        // Limit content to first 1000 lines to avoid overloading the model
        let content = file.content.lines()
            .take(1000)
            .collect::<Vec<&str>>()
            .join("\n");
        
        // Format file info
        format!(
            "File: {}\nLanguage: {}\n\n```{}\n{}\n```\n",
            file.path,
            file.language,
            file.language.to_lowercase(),
            content
        )
    }
    
    // Create the AI prompt for batch analysis
    fn create_batch_analysis_prompt(&self, directory: &str, file_texts: &[String]) -> String {
        format!(
            "You are an expert software developer analyzing a codebase. 
Below are files from the same directory or related functionality in a project. 
Analyze these files and provide a concise summary (3-5 paragraphs) about: 
1. What functionality this code provides 
2. How the files relate to each other 
3. Key classes, functions and interfaces 
4. Design patterns or architectural approaches used 
5. Any notable algorithms or techniques 

Be specific about what the code does but don't waste words simply listing files. 
Focus on explaining the overall purpose and functionality.

Directory: {}\n\n{}",
            directory,
            file_texts.join("\n\n")
        )
    }
    
    // Process the result of an AI batch analysis
    async fn process_batch_result(
        &self, 
        model: &Box<dyn AiModel>, 
        prompt: &str, 
        summaries: &mut Vec<String>,
        batch_index: usize, 
        total_batches: usize, 
        file_count: usize, 
        batch_desc: &str
    ) {
        match model.generate_response(prompt).await {
            Ok(text) => {
                summaries.push(text);
                self.log_batch_success(batch_index, total_batches, file_count, batch_desc);
            },
            Err(e) => {
                self.log_batch_failure(batch_index, total_batches, &e);
            }
        }
    }
    
    // Log successful batch processing
    fn log_batch_success(&self, batch_index: usize, total_batches: usize, file_count: usize, batch_desc: &str) {
        style::print_info(&format!(
            "✅ [Batch {}/{}] Successfully summarized {} files from {}", 
            batch_index + 1, 
            total_batches, 
            file_count,
            batch_desc
        ));
    }
    
    // Log failed batch processing
    fn log_batch_failure(&self, batch_index: usize, total_batches: usize, error: &dyn std::fmt::Display) {
        style::print_warning(&format!(
            "❌ [Batch {}/{}] Failed to summarize batch: {}", 
            batch_index + 1, 
            total_batches, 
            error
        ));
    }
    
    /// Generate the final description using the high-tier AI model
    async fn generate_final_description(&self, batch_summaries: &[String]) -> AppResult<String> {
        // Create the high-tier AI model
        style::print_info("📚 Creating high-tier AI model for final analysis...");
        let high_tier_model = factory::create_ai_model(self.ai_config.clone(), ModelTier::High)
            .map_err(|e| AppError::Ai(e))?;
        
        // Join all the summaries
        let all_summaries = batch_summaries.join("\n\n---\n\n");
        
        // Create the prompt
        let prompt = format!(
            "You are an expert software developer creating a clear overview of a codebase 
based on the following summarized components. Each separated section represents 
the analysis of different parts of the codebase.

Generate a comprehensive but concise description of this project that includes:

1. An overview of the project's purpose and functionality
2. The main components and how they interact
3. The architecture and design patterns used
4. Key technologies and libraries leveraged
5. Notable algorithms or techniques implemented

IMPORTANT: Do NOT invent or suggest a name for the codebase. Only use a name if you 
see it clearly mentioned in the code itself (such as in package names, documentation, 
or code comments). If you don't find a definitive name, simply refer to it as 'this project'
or 'this codebase'.

Format the description with clear sections and focus on providing a high-level 
understanding that would be useful for new developers joining the project.

Here are the component summaries:

{}",
            all_summaries
        );
        
        style::print_info("🧠 Generating final codebase description with high-tier AI model...");
        style::print_info("⏳ This may take a moment as the AI analyzes all component summaries...");
        
        // Generate the final description
        let description = match high_tier_model.generate_response(&prompt).await {
            Ok(text) => {
                style::print_info("✅ Successfully generated comprehensive codebase description!");
                text
            },
            Err(e) => {
                let error = AppError::Ai(e);
                style::print_warning(&format!("❌ Failed to generate final description: {}", error));
                return Err(error);
            }
        };
        
        Ok(description)
    }
}