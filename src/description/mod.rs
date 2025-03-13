use std::path::Path;
use std::sync::{Arc, Mutex};
use std::fs;
use std::collections::HashMap;

use ignore::{DirEntry, WalkBuilder};
use rayon::prelude::*;

use crate::ai::{AiConfig, ModelTier, factory};
use crate::cache::AnalysisCache;
use crate::metrics::language::LanguageDetector;
use crate::output::style;
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
    fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
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
    pub async fn describe_codebase<P: AsRef<Path>>(&self, dir_path: P) -> Result<String, String> {
        let path = dir_path.as_ref();
        
        if !path.exists() {
            return Err(format!("Path '{}' does not exist", path.display()));
        }

        if !path.is_dir() {
            return Err(format!("Path '{}' is not a directory", path.display()));
        }
        
        style::print_info("🔎 Scanning codebase files...");
        
        // Collect and process files
        let batches = self.collect_files_internal(path)?;
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
    pub fn collect_files<P: AsRef<Path>>(&self, dir_path: P) -> Result<Vec<FileBatch>, String> {
        self.collect_files_internal(dir_path)
    }

    // Internal implementation for file collection
    fn collect_files_internal<P: AsRef<Path>>(&self, dir_path: P) -> Result<Vec<FileBatch>, String> {
        self.collect_files_impl(dir_path)
    }

    // Actual implementation
    fn collect_files_impl<P: AsRef<Path>>(&self, dir_path: P) -> Result<Vec<FileBatch>, String> {
        let path = dir_path.as_ref();
        
        // Create walker that respects .gitignore
        let walker = WalkBuilder::new(path)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .filter_entry(|e| {
                let path_str = e.path().to_string_lossy();
                let file_name = e.path().file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                !path_str.contains("/.git/") && 
                !path_str.ends_with(".lock") && 
                !path_str.ends_with(".gitignore") &&
                !path_str.ends_with(".png") &&
                !path_str.ends_with(".jpg") &&
                !path_str.ends_with(".jpeg") &&
                !path_str.ends_with(".gif") &&
                !path_str.ends_with(".svg") &&
                !path_str.ends_with(".woff") &&
                !path_str.ends_with(".woff2") &&
                !path_str.ends_with(".ttf") &&
                !path_str.ends_with(".eot") &&
                !path_str.ends_with(".ico") &&
                file_name != ".DS_Store"
            })
            .build();
        
        // Collect all entries first
        let entries: Vec<DirEntry> = walker
            .filter_map(|result| {
                match result {
                    Ok(entry) => Some(entry),
                    Err(err) => {
                        style::print_warning(&format!("Warning: {}", err));
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
    async fn generate_batch_summaries(&self, batches: &[FileBatch]) -> Result<Vec<String>, String> {
        let mut summaries = Vec::new();
        
        // Create the low-tier AI model
        let low_tier_model = factory::create_ai_model(self.ai_config.clone(), ModelTier::Low)
            .map_err(|e| format!("Failed to create AI model: {}", e))?;
        
        style::print_info("Generating batch summaries with AI...");
        style::print_info(&format!("Processing {} batch(es) with low-tier model", batches.len()));
        
        // We'll process each batch sequentially, regardless of the parallel flag
        // This is to avoid async runtime issues
        for (batch_index, batch) in batches.iter().enumerate() {
            // Format batch description for logging
            let batch_desc = if batch.base_path.is_empty() {
                "root directory".to_string()
            } else {
                batch.base_path.clone()
            };
            let file_count = batch.files.len();
            
            // Log batch beginning
            style::print_info(&format!("🔍 [Batch {}/{}] Processing {} files from {}", 
                batch_index + 1, 
                batches.len(), 
                file_count,
                batch_desc
            ));
            
            // Prepare the files for the prompt
            let mut file_texts = Vec::new();
            
            for file in &batch.files {
                // Limit content to first 1000 lines to avoid overloading the model
                let content = file.content.lines()
                    .take(1000)
                    .collect::<Vec<&str>>()
                    .join("\n");
                
                // Format file info
                let file_text = format!(
                    "File: {}\nLanguage: {}\n\n```{}\n{}\n```\n",
                    file.path,
                    file.language,
                    file.language.to_lowercase(),
                    content
                );
                
                file_texts.push(file_text);
            }
            
            // Create the prompt
            let prompt = format!(
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
                batch.base_path,
                file_texts.join("\n\n")
            );
            
            // Call the AI model
            match low_tier_model.generate_response(&prompt).await {
                Ok(text) => {
                    summaries.push(text);
                    // Log successful completion
                    style::print_info(&format!("✅ [Batch {}/{}] Successfully summarized {} files from {}", 
                        batch_index + 1, 
                        batches.len(), 
                        file_count,
                        batch_desc
                    ));
                },
                Err(e) => {
                    style::print_warning(&format!("❌ [Batch {}/{}] Failed to summarize batch: {}", 
                        batch_index + 1, 
                        batches.len(), 
                        e
                    ));
                }
            }
        }
        
        Ok(summaries)
    }
    
    /// Generate the final description using the high-tier AI model
    async fn generate_final_description(&self, batch_summaries: &[String]) -> Result<String, String> {
        // Create the high-tier AI model
        style::print_info("📚 Creating high-tier AI model for final analysis...");
        let high_tier_model = factory::create_ai_model(self.ai_config.clone(), ModelTier::High)
            .map_err(|e| format!("Failed to create AI model: {}", e))?;
        
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
                let error_msg = format!("Failed to generate final description: {}", e);
                style::print_warning(&format!("❌ {}", error_msg));
                return Err(error_msg);
            }
        };
        
        Ok(description)
    }
}