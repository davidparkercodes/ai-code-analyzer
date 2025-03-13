use crate::output::style;
use crate::util::error::{AppError, AppResult, handle_command_error, AppErrorType, to_app_error};
use crate::util::file_filter::FileFilter;
use crate::util::parallel::{log_parallel_status, parse_parallel_flag};
use std::time::Instant;
use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use regex::Regex;

pub fn execute(path: String, output_dir: Option<String>, no_parallel: bool) -> i32 {
    match execute_clean_comments_command(path, output_dir, no_parallel) {
        Ok(_) => 0,
        Err(error) => handle_command_error(&error)
    }
}

fn execute_clean_comments_command(
    path: String, 
    output_dir: Option<String>,
    no_parallel: bool
) -> AppResult<()> {
    let parallel_enabled = parse_parallel_flag(no_parallel);
    
    display_clean_header(&path);
    log_parallel_status(parallel_enabled);
    
    let start_time = Instant::now();
    let stats = clean_comments(&path, output_dir.as_deref())?;
    
    display_clean_results(stats, start_time);
    
    Ok(())
}

fn display_clean_header(directory_path: &str) {
    style::print_header("Cleaning Double-Slash Comments");
    style::print_info(&format!("Cleaning Rust files in directory: {}", directory_path));
}

struct CleanStats {
    processed_files: usize,
    changed_files: usize,
    removed_comments: usize,
}

fn clean_comments(directory_path: &str, output_dir: Option<&str>) -> AppResult<CleanStats> {
    let path = Path::new(directory_path);
    
    if !path.exists() {
        return Err(AppError::FileSystem { 
            path: path.to_path_buf(), 
            message: "Path does not exist".to_string() 
        });
    }
    
    let mut stats = CleanStats {
        processed_files: 0,
        changed_files: 0,
        removed_comments: 0,
    };
    
    // Regular expression to match double-slash comments
    let comment_regex = Regex::new(r"//.+$").map_err(|e| {
        to_app_error(format!("Failed to compile regex: {}", e), AppErrorType::Internal)
    })?;
    
    // Create output directory if specified
    let output_base = match output_dir {
        Some(dir) => {
            let out_path = Path::new(dir);
            if !out_path.exists() {
                fs::create_dir_all(out_path).map_err(|e| {
                    AppError::FileSystem { 
                        path: out_path.to_path_buf(), 
                        message: format!("Failed to create output directory: {}", e) 
                    }
                })?;
            }
            Some(PathBuf::from(dir))
        },
        None => None,
    };
    
    // Handle both file and directory paths
    if path.is_file() {
        // Single file processing
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(content) = fs::read_to_string(path) {
                stats.processed_files += 1;
                
                let mut comment_count = 0;
                let cleaned_content = clean_file_content(&content, &comment_regex, &mut comment_count);
                
                // Only write if comments were found and removed
                if comment_count > 0 {
                    stats.changed_files += 1;
                    stats.removed_comments += comment_count;
                    
                    // Determine target path
                    let file_name = path.file_name().unwrap();
                    let target_path = match &output_base {
                        Some(base) => base.join(file_name),
                        None => path.to_path_buf(),
                    };
                    
                    // Write cleaned content
                    fs::write(&target_path, cleaned_content).map_err(|e| {
                        AppError::FileSystem { 
                            path: target_path.clone(), 
                            message: format!("Failed to write file: {}", e) 
                        }
                    })?;
                }
            }
        }
    } else {
        // Directory processing - walk through all files
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();
            
            // Skip directories and non-Rust files
            if !file_path.is_file() || file_path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            
            // Skip system files and binary files
            if FileFilter::should_exclude(file_path) {
                continue;
            }
            
            if let Ok(content) = fs::read_to_string(file_path) {
                stats.processed_files += 1;
                
                let mut comment_count = 0;
                let cleaned_content = clean_file_content(&content, &comment_regex, &mut comment_count);
                
                // Only process files with comments
                if comment_count > 0 {
                    stats.changed_files += 1;
                    stats.removed_comments += comment_count;
                    
                    // Determine where to write the file
                    let target_path = match &output_base {
                        Some(base) => {
                            // Create a relative path from the input directory and append to output directory
                            let rel_path = file_path.strip_prefix(path).unwrap_or(file_path);
                            let target = base.join(rel_path);
                            
                            // Ensure parent directories exist
                            if let Some(parent) = target.parent() {
                                fs::create_dir_all(parent).map_err(|e| {
                                    AppError::FileSystem { 
                                        path: parent.to_path_buf(), 
                                        message: format!("Failed to create directory: {}", e) 
                                    }
                                })?;
                            }
                            
                            target
                        },
                        None => file_path.to_path_buf(),
                    };
                    
                    // Write the cleaned content
                    fs::write(&target_path, cleaned_content).map_err(|e| {
                        AppError::FileSystem { 
                            path: target_path.clone(), 
                            message: format!("Failed to write file: {}", e) 
                        }
                    })?;
                }
            }
        }
    }
    
    Ok(stats)
}

fn clean_file_content(content: &str, comment_regex: &Regex, comment_count: &mut usize) -> String {
    let mut result = String::with_capacity(content.len());
    
    for line in content.lines() {
        // Skip triple-slash comments (documentation comments)
        if line.trim_start().starts_with("///") {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        
        // Handle double-slash comments
        if let Some(caps) = comment_regex.captures(line) {
            *comment_count += 1;
            
            // Get the matched range for the comment
            let mat = caps.get(0).unwrap();
            let clean_line = &line[0..mat.start()];
            
            // Only add non-empty lines to result
            if !clean_line.trim().is_empty() {
                result.push_str(clean_line);
                result.push('\n');
            } else {
                // Skip lines that only contained comments
                continue;
            }
        } else {
            // No comment in this line, add it unchanged
            result.push_str(line);
            result.push('\n');
        }
    }
    
    result
}

fn display_clean_results(stats: CleanStats, start_time: Instant) {
    let elapsed = start_time.elapsed();
    
    style::print_header("\nComment Cleaning Complete");
    println!("Files processed: {}", stats.processed_files);
    println!("Files changed: {}", stats.changed_files);
    println!("Comments removed: {}", stats.removed_comments);
    style::print_success(&format!("Cleaning completed in {:.2?}", elapsed));
}