use crate::output::style;
use crate::util::error::{AppError, AppResult, handle_command_error, AppErrorType, to_app_error};
use crate::util::file_filter::FileFilter;
use crate::util::parallel::{log_parallel_status, parse_parallel_flag};
use std::time::Instant;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use walkdir::WalkDir;
use regex::Regex;

pub fn execute(
    path: String, 
    output_dir: Option<String>, 
    no_parallel: bool,
    no_git: bool,
    force: bool,
    dry_run: bool
) -> i32 {
    match execute_clean_comments_command(path, output_dir, no_parallel, no_git, force, dry_run) {
        Ok(_) => 0,
        Err(error) => handle_command_error(&error)
    }
}

fn execute_clean_comments_command(
    path: String, 
    output_dir: Option<String>,
    no_parallel: bool,
    no_git: bool,
    force: bool,
    dry_run: bool
) -> AppResult<()> {
    let parallel_enabled = parse_parallel_flag(no_parallel);
    let path_buf = PathBuf::from(&path);
    
    // If dry-run, don't need Git repository check or confirmation
    if !dry_run {
        // Git repository check
        let is_git_repo = if !no_git {
            check_git_repository(&path_buf)?
        } else {
            false
        };
        
        // If not a git repo and not in force mode, ask for confirmation
        if !is_git_repo && !force && output_dir.is_none() && !confirm_non_git_operation()? {
            style::print_info("Operation cancelled by user.");
            return Ok(());
        }
    } else {
        style::print_info("Running in dry-run mode. No files will be modified.");
    }
    
    display_clean_header(&path);
    log_parallel_status(parallel_enabled);
    
    let start_time = Instant::now();
    let stats = clean_comments(&path, output_dir.as_deref(), dry_run)?;
    
    display_clean_results(&stats, start_time);
    
    // Handle Git operations if this is a git repository and changes were made
    if !dry_run && no_git == false && stats.changed_files > 0 && output_dir.is_none() {
        let is_git_repo = check_git_repository(&path_buf)?;
        if is_git_repo {
            handle_git_operations(&path_buf)?;
        }
    }
    
    Ok(())
}

fn check_git_repository(path: &Path) -> AppResult<bool> {
    // Check if the directory is a git repository
    let mut git_dir = path.to_path_buf();
    
    // If the path is a file, use its parent directory
    if path.is_file() {
        if let Some(parent) = path.parent() {
            git_dir = parent.to_path_buf();
        }
    }
    
    // Try to run git status to check if it's a git repository
    let status = Command::new("git")
        .arg("-C")
        .arg(&git_dir)
        .arg("status")
        .output();
    
    match status {
        Ok(output) => Ok(output.status.success()),
        Err(_) => {
            // Git command failed - either git is not installed or not a repository
            style::print_info("Git is not available or this is not a git repository.");
            Ok(false)
        }
    }
}

fn confirm_non_git_operation() -> AppResult<bool> {
    style::print_warning("No git repository detected. Changes will be made directly to your files.");
    style::print_warning("Are you sure you want to continue? (y/N): ");
    
    // Flush to ensure the prompt is displayed
    io::stdout().flush().map_err(AppError::Io)?;
    
    let mut response = String::new();
    io::stdin().read_line(&mut response).map_err(AppError::Io)?;
    
    let response = response.trim().to_lowercase();
    Ok(response == "y" || response == "yes")
}

fn handle_git_operations(path: &Path) -> AppResult<()> {
    style::print_header("\nGit Operations");
    
    // Get a list of modified files
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("status")
        .arg("--porcelain")
        .output()
        .map_err(AppError::Io)?;
    
    let modified_files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.starts_with(" M") || line.starts_with("M"))
        .map(|line| line[3..].to_string())
        .collect::<Vec<String>>();
    
    if modified_files.is_empty() {
        style::print_info("No files were modified that need to be committed.");
        return Ok(());
    }
    
    // Add the modified files to git
    style::print_info(&format!("Adding {} modified files to git...", modified_files.len()));
    
    let mut git_add = Command::new("git");
    git_add.arg("-C").arg(path).arg("add");
    
    for file in &modified_files {
        git_add.arg(file);
    }
    
    let add_output = git_add.output().map_err(AppError::Io)?;
    
    if !add_output.status.success() {
        return Err(to_app_error(
            format!("Git add failed: {}", String::from_utf8_lossy(&add_output.stderr)),
            AppErrorType::Internal
        ));
    }
    
    // Create a commit
    style::print_info("Creating commit...");
    
    let commit_output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("commit")
        .arg("-m")
        .arg("Cleaning up unnecessary comments")
        .output()
        .map_err(AppError::Io)?;
    
    if !commit_output.status.success() {
        return Err(to_app_error(
            format!("Git commit failed: {}", String::from_utf8_lossy(&commit_output.stderr)),
            AppErrorType::Internal
        ));
    }
    
    style::print_success("Successfully committed changes to git repository.");
    
    Ok(())
}

fn display_clean_header(directory_path: &str) {
    style::print_header("Cleaning Double-Slash Comments");
    style::print_info(&format!("Analyzing Rust files in directory: {}", directory_path));
}

struct CleanStats {
    processed_files: usize,
    changed_files: usize,
    removed_comments: usize,
}

fn clean_comments(directory_path: &str, output_dir: Option<&str>, dry_run: bool) -> AppResult<CleanStats> {
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
    
    // Regular expression to detect ignore pattern
    let ignore_regex = Regex::new(r"//.*aicodeanalyzer:\s*ignore").map_err(|e| {
        to_app_error(format!("Failed to compile regex: {}", e), AppErrorType::Internal)
    })?;
    
    // Create output directory if specified and not in dry-run mode
    let output_base = if !dry_run {
        match output_dir {
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
        }
    } else {
        None
    };
    
    // Handle both file and directory paths
    if path.is_file() {
        // Single file processing
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(content) = fs::read_to_string(path) {
                stats.processed_files += 1;
                
                let mut comment_count = 0;
                let cleaned_content = clean_file_content(&content, &comment_regex, &ignore_regex, &mut comment_count);
                
                // Only process if comments were found and removed
                if comment_count > 0 {
                    stats.changed_files += 1;
                    stats.removed_comments += comment_count;
                    
                    if dry_run {
                        // In dry-run mode, display what would be changed
                        style::print_info(&format!("Would remove {} comments from {}", comment_count, path.display()));
                        print_comment_preview(&content, &cleaned_content, path.to_str().unwrap_or("file"));
                    } else {
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
                let cleaned_content = clean_file_content(&content, &comment_regex, &ignore_regex, &mut comment_count);
                
                // Only process files with comments
                if comment_count > 0 {
                    stats.changed_files += 1;
                    stats.removed_comments += comment_count;
                    
                    if dry_run {
                        // In dry-run mode, display what would be changed
                        style::print_info(&format!("Would remove {} comments from {}", comment_count, file_path.display()));
                        print_comment_preview(&content, &cleaned_content, file_path.to_str().unwrap_or("file"));
                    } else {
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
    }
    
    Ok(stats)
}

fn clean_file_content(
    content: &str, 
    comment_regex: &Regex, 
    ignore_regex: &Regex, 
    comment_count: &mut usize
) -> String {
    let mut result = String::with_capacity(content.len());
    
    for line in content.lines() {
        // Skip triple-slash comments (documentation comments)
        if line.trim_start().starts_with("///") {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        
        // Check if this line has the ignore marker
        if ignore_regex.is_match(line) {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        
        // Parse the line to find string literals and only remove comments outside of them
        let processed_line = if let Some(processed) = process_line_preserving_strings(line, comment_regex, comment_count) {
            processed
        } else {
            line.to_string()
        };
        
        // Add the processed line if it's not empty
        if !processed_line.trim().is_empty() {
            result.push_str(&processed_line);
            result.push('\n');
        }
    }
    
    result
}

fn display_clean_results(stats: &CleanStats, start_time: Instant) {
    let elapsed = start_time.elapsed();
    
    style::print_header("\nComment Cleaning Complete");
    println!("Files processed: {}", stats.processed_files);
    println!("Files changed: {}", stats.changed_files);
    println!("Comments removed: {}", stats.removed_comments);
    style::print_success(&format!("Cleaning completed in {:.2?}", elapsed));
}

/// Process a line of code, preserving string literals while removing comments outside them
fn process_line_preserving_strings(line: &str, _comment_regex: &Regex, comment_count: &mut usize) -> Option<String> {
    // State tracking for string parsing
    let mut in_string = false;
    let mut in_raw_string = false;
    let mut escape_next = false;
    let chars = line.chars().collect::<Vec<_>>();
    let length = chars.len();
    
    // Find potential comment positions that are outside string literals
    let mut comment_pos = None;
    
    for i in 0..length {
        let c = chars[i];
        
        if escape_next {
            // Skip escaped character
            escape_next = false;
            continue;
        }
        
        match c {
            // Handle escape sequences inside strings
            '\\' if in_string => {
                escape_next = true;
            },
            
            // Handle string literals with double quotes
            '"' => {
                // Handle raw strings (r#"..."#)
                if i > 0 && chars[i-1] == 'r' && !in_string && !in_raw_string {
                    in_raw_string = true;
                    continue;
                }
                
                // Toggle normal string state if not in a raw string
                if !in_raw_string {
                    in_string = !in_string;
                }
            },
            
            // Handle closing of raw strings
            '#' if in_raw_string && i > 0 && chars[i-1] == '"' => {
                in_raw_string = false;
            },
            
            // Detect comments outside of string literals
            '/' if i + 1 < length && chars[i+1] == '/' && !in_string && !in_raw_string => {
                comment_pos = Some(i);
                break;
            },
            
            _ => {}
        }
    }
    
    // If we found a comment outside string literals, remove it
    if let Some(pos) = comment_pos {
        *comment_count += 1;
        let cleaned = line[0..pos].trim_end();
        
        if cleaned.is_empty() {
            return None; // Entire line was a comment
        }
        
        return Some(cleaned.to_string());
    }
    
    // No comments found or they were inside string literals
    None
}

fn print_comment_preview(original: &str, cleaned: &str, file_path: &str) {
    // Find the differences between original and cleaned content
    let original_lines: Vec<&str> = original.lines().collect();
    let cleaned_lines: Vec<&str> = cleaned.lines().collect();
    
    // For each original line, check if it's in the cleaned content
    // If not, it was likely a comment-only line that was removed
    // If it's different, it likely had a comment at the end that was removed
    
    const MAX_PREVIEW_LINES: usize = 5; // Limit preview to first 5 changes
    let mut preview_count = 0;
    
    println!("\n{}", style::bold(&format!("Preview of changes for {}", file_path)));
    
    // Compare up to the smaller of the two line counts
    let min_lines = std::cmp::min(original_lines.len(), cleaned_lines.len());
    
    for i in 0..min_lines {
        // If lines differ, show the difference
        if original_lines[i] != cleaned_lines[i] && preview_count < MAX_PREVIEW_LINES {
            let original_line = original_lines[i];
            let cleaned_line = cleaned_lines[i];
            
            if cleaned_line.trim().is_empty() {
                // Line was completely removed (was only a comment)
                println!("- {}", style::dimmed(original_line));
            } else {
                // Part of the line was removed (comment at the end)
                println!("- {}", style::dimmed(original_line));
                println!("+ {}", style::success(cleaned_line));
            }
            
            preview_count += 1;
        }
    }
    
    // Check for lines completely removed (original had more lines than cleaned)
    if original_lines.len() > cleaned_lines.len() {
        for i in min_lines..original_lines.len() {
            if preview_count < MAX_PREVIEW_LINES {
                println!("- {}", style::dimmed(original_lines[i]));
                preview_count += 1;
            } else {
                break;
            }
        }
    }
    
    // If there are more changes than the preview limit, show a message
    let total_changes = original_lines.len() - cleaned_lines.len() + 
                      (0..min_lines).filter(|&i| original_lines[i] != cleaned_lines[i]).count();
    
    if total_changes > MAX_PREVIEW_LINES {
        println!("... and {} more changes", total_changes - MAX_PREVIEW_LINES);
    }
    
    println!();
}