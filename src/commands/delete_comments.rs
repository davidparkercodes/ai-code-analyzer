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
use chrono;
use serde::{Serialize, Deserialize};
use serde_json;

/// Record of a deleted comment for JSON output
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeletedComment {
    /// File path where the comment was removed
    file: String,
    /// Line number where the comment was removed
    line: usize,
    /// The actual comment text that was removed
    comment_removed: String,
}

pub fn execute(
    path: String, 
    language: String, 
    no_output: bool,
    output_path: Option<String>, 
    no_parallel: bool,
    no_git: bool,
    _force: bool,
    dry_run: bool
) -> i32 {
    match execute_delete_comments_command(path, language, no_output, output_path, no_parallel, no_git, _force, dry_run) {
        Ok(_) => 0,
        Err(error) => handle_command_error(&error)
    }
}

fn execute_delete_comments_command(
    path: String, 
    language: String,
    no_output: bool,
    output_path: Option<String>,
    no_parallel: bool,
    no_git: bool,
    _force: bool,
    dry_run: bool
) -> AppResult<()> {
    if !["rust", "python"].contains(&language.to_lowercase().as_str()) {
        return Err(to_app_error(
            format!("Language '{}' is not supported. Currently only 'rust' and 'python' are supported.", language),
            AppErrorType::Internal
        ));
    }
    let parallel_enabled = parse_parallel_flag(no_parallel);
    let path_buf = PathBuf::from(&path);
    
    let is_git_repo = if !no_git {
        check_git_repository(&path_buf)?
    } else {
        false
    };
    
    if !is_git_repo && !dry_run && (output_path.is_none() && !no_output) {
        style::print_error("This command requires a git repository to run safely.");
        style::print_error("Please run in a git repository, use --dry-run, or specify an output directory.");
        return Err(to_app_error(
            "Git repository required for this operation.".to_string(),
            AppErrorType::Internal
        ));
    }
    
    if is_git_repo && !dry_run && !no_git {
        style::print_header("\n🔄 Git Operations");
        style::print_info("ℹ️ This command will:");
        style::print_info("1️⃣ Create a new branch for the changes");
        style::print_info("2️⃣ Delete comments from your code files");
        style::print_info("3️⃣ Commit the changes to the new branch");
        style::print_info("4️⃣ Push the branch to your remote repository");
        style::print_info("5️⃣ Create a PR for review (if GitHub CLI is available)");
    }
    
    if !dry_run && !confirm_operation(is_git_repo)? {
        style::print_info("❌ Operation cancelled by user.");
        return Ok(());
    }
    
    if is_git_repo && !dry_run && !no_git {
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let branch_name = format!("ai-code-analyzer/delete-comments-{}-{}", language, timestamp);
        create_git_branch(&path_buf, &branch_name)?;
    }
    
    if dry_run {
        style::print_info("🔍 Running in dry-run mode. No files will be modified.");
    }
    
    display_delete_header(&path, &language);
    log_parallel_status(parallel_enabled);
    
    let start_time = Instant::now();
    let effective_output_dir = if no_output {
        None
    } else {
        output_path.as_deref()
    };
    
    let stats = delete_comments(&path, &language, effective_output_dir, dry_run)?;
    
    display_delete_results(&stats, start_time);
    
    // Export the JSON output if there are comments to report
    if stats.removed_comments > 0 {
        export_json_results(&stats, &path)?;
    }
    
    if !dry_run && no_git == false && stats.changed_files > 0 && effective_output_dir.is_none() {
        let is_git_repo = check_git_repository(&path_buf)?;
        if is_git_repo {
            handle_git_operations(&path_buf)?;
        }
    }
    
    Ok(())
}

fn check_git_repository(path: &Path) -> AppResult<bool> {
    let mut git_dir = path.to_path_buf();
    
    if path.is_file() {
        if let Some(parent) = path.parent() {
            git_dir = parent.to_path_buf();
        }
    }
    
    let status = Command::new("git")
        .arg("-C")
        .arg(&git_dir)
        .arg("status")
        .output();
    
    match status {
        Ok(output) => Ok(output.status.success()),
        Err(_) => {
            style::print_info("Git is not available or this is not a git repository.");
            Ok(false)
        }
    }
}

fn confirm_operation(is_git_repo: bool) -> AppResult<bool> {
    if is_git_repo {
        style::print_warning("⚠️ Are you sure you want to delete comments from your code? (y/N): ");
    } else {
        style::print_warning("⚠️ No git repository detected. Changes will be made to a separate output directory.");
        style::print_warning("⚠️ Are you sure you want to continue? (y/N): ");
    }
    
    io::stdout().flush().map_err(AppError::Io)?;
    
    let mut response = String::new();
    io::stdin().read_line(&mut response).map_err(AppError::Io)?;
    
    let response = response.trim().to_lowercase();
    Ok(response == "y" || response == "yes")
}

fn create_git_branch(path: &Path, branch_name: &str) -> AppResult<()> {
    style::print_info(&format!("🔄 Creating git branch: {}", branch_name));
    
    let branch_check = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("show-ref")
        .arg("--verify")
        .arg(&format!("refs/heads/{}", branch_name))
        .output()
        .map_err(AppError::Io)?;
    
    if branch_check.status.success() {
        style::print_warning(&format!("⚠️ Branch '{}' already exists. Using existing branch.", branch_name));
        
        let checkout_output = Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("checkout")
            .arg(branch_name)
            .output()
            .map_err(AppError::Io)?;
        
        if !checkout_output.status.success() {
            return Err(to_app_error(
                format!("Failed to checkout branch: {}", String::from_utf8_lossy(&checkout_output.stderr)),
                AppErrorType::Internal
            ));
        }
    } else {
        let branch_output = Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("checkout")
            .arg("-b")
            .arg(branch_name)
            .output()
            .map_err(AppError::Io)?;
        
        if !branch_output.status.success() {
            return Err(to_app_error(
                format!("Failed to create branch: {}", String::from_utf8_lossy(&branch_output.stderr)),
                AppErrorType::Internal
            ));
        }
    }
    
    style::print_success(&format!("✅ Now working on branch: {}", branch_name));
    Ok(())
}

fn handle_git_operations(path: &Path) -> AppResult<()> {
    style::print_header("\n🔄 Git Operations");
    
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
        style::print_warning("⚠️ No files were modified that need to be committed.");
        style::print_info("ℹ️ No PR will be created since there are no changes.");
        return Ok(());
    }
    
    style::print_info(&format!("📂 Adding {} modified files to git...", modified_files.len()));
    
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
    
    style::print_info("📝 Creating commit...");
    
    let commit_output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("commit")
        .arg("-m")
        .arg("Deleting unnecessary comments")
        .output()
        .map_err(AppError::Io)?;
    
    if !commit_output.status.success() {
        return Err(to_app_error(
            format!("Git commit failed: {}", String::from_utf8_lossy(&commit_output.stderr)),
            AppErrorType::Internal
        ));
    }
    
    style::print_success("✅ Successfully committed changes to git repository.");
    
    style::print_info("⬆️ Pushing changes to remote repository...");
    
    let branch_output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .map_err(AppError::Io)?;
    
    if !branch_output.status.success() {
        return Err(to_app_error(
            format!("Failed to get branch name: {}", String::from_utf8_lossy(&branch_output.stderr)),
            AppErrorType::Internal
        ));
    }
    
    let branch_name = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
    
    let push_output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("push")
        .arg("--set-upstream")
        .arg("origin")
        .arg(&branch_name)
        .output()
        .map_err(AppError::Io)?;
    
    if !push_output.status.success() {
        return Err(to_app_error(
            format!("Git push failed: {}", String::from_utf8_lossy(&push_output.stderr)),
            AppErrorType::Internal
        ));
    }
    
    style::print_success("✅ Successfully pushed changes to remote repository.");
    
    style::print_info("🔄 Creating pull request...");
    
    let gh_check = Command::new("which")
        .arg("gh")
        .output();
    
    if gh_check.is_err() || !gh_check.unwrap().status.success() {
        style::print_warning("⚠️ GitHub CLI not found. Skipping PR creation.");
        style::print_info("ℹ️ To create a PR manually, visit your repository on GitHub.");
        return Ok(());
    }
    
    let pr_title = format!("Delete comments from codebase");
    let pr_body = format!("## Summary\n- Removed unnecessary comments from codebase\n- Improved code readability\n\n## Changes\n- Deleted single-line comments\n- Preserved documentation comments\n- Maintained code functionality");
    
    let pr_output = Command::new("gh")
        .arg("pr")
        .arg("create")
        .arg("--title")
        .arg(pr_title)
        .arg("--body")
        .arg(pr_body)
        .current_dir(path)
        .output()
        .map_err(AppError::Io)?;
    
    if !pr_output.status.success() {
        style::print_warning(&format!("⚠️ PR creation failed: {}", String::from_utf8_lossy(&pr_output.stderr)));
        style::print_info("ℹ️ You can create a PR manually through the GitHub website.");
        return Ok(());
    }
    
    let pr_url = String::from_utf8_lossy(&pr_output.stdout).trim().to_string();
    style::print_success(&format!("✅ Successfully created PR: {}", pr_url));
    
    Ok(())
}

fn display_delete_header(directory_path: &str, language: &str) {
    style::print_header(&format!("🗑️ Deleting Comments from {} Files", language.to_uppercase()));
    style::print_info(&format!("🔍 Analyzing {} files in directory: {}", language, directory_path));
}

struct DeleteStats {
    processed_files: usize,
    changed_files: usize,
    removed_comments: usize,
    /// Records of all deleted comments for JSON output
    deleted_comments: Vec<DeletedComment>,
}

fn delete_comments(directory_path: &str, language: &str, output_dir: Option<&str>, dry_run: bool) -> AppResult<DeleteStats> {
    let path = Path::new(directory_path);
    
    if !path.exists() {
        return Err(AppError::FileSystem { 
            path: path.to_path_buf(), 
            message: "Path does not exist".to_string() 
        });
    }
    
    let mut stats = DeleteStats {
        processed_files: 0,
        changed_files: 0,
        removed_comments: 0,
        deleted_comments: Vec::new(),
    };
    
    let (file_extension, comment_pattern, doc_comment_prefix, ignore_pattern) = match language.to_lowercase().as_str() {
        "rust" => (
            "rs",
            r"//.+$",
            "///",
            r"//.*aicodeanalyzer:\s*ignore"
        ),
        "python" => (
            "py",
            r"#.+$",
            "###",
            r"#.*aicodeanalyzer:\s*ignore"
        ),
        _ => {
            return Err(to_app_error(
                format!("Language '{}' is not supported.", language),
                AppErrorType::Internal
            ));
        }
    };
    
    let comment_regex = Regex::new(comment_pattern).map_err(|e| {
        to_app_error(format!("Failed to compile regex: {}", e), AppErrorType::Internal)
    })?;
    
    let ignore_regex = Regex::new(ignore_pattern).map_err(|e| {
        to_app_error(format!("Failed to compile regex: {}", e), AppErrorType::Internal)
    })?;
    
    // Pattern to match and ignore (already updated with new name)
    let ignore_pattern_str = r"aicodeanalyzer:\s*ignore";
    
    let output_base = match output_dir {
        Some(dir) => {
            if dir.starts_with('/') {
                let out_path = Path::new(&dir);
                if !out_path.exists() {
                    fs::create_dir_all(out_path).map_err(|e| {
                        AppError::FileSystem { 
                            path: out_path.to_path_buf(), 
                            message: format!("Failed to create output directory: {}", e) 
                        }
                    })?;
                }
                Some(PathBuf::from(dir))
            } else {
                let base_path = crate::output::path::ensure_base_output_dir()?;
                let date_path = crate::output::path::ensure_date_subdirectory(&base_path)?;
                let delete_comments_path = crate::output::path::ensure_command_subdirectory(&date_path, "delete_comments")?;
                let final_dir = delete_comments_path.join(dir);
                
                if !final_dir.exists() {
                    fs::create_dir_all(&final_dir).map_err(|e| {
                        AppError::FileSystem { 
                            path: final_dir.clone(), 
                            message: format!("Failed to create output directory: {}", e) 
                        }
                    })?;
                }
                
                Some(final_dir)
            }
        },
        None => {
            if dry_run {
                None
            } else {
                None
            }
        }
    };
    
    if path.is_file() {
        if path.extension().and_then(|e| e.to_str()) == Some(file_extension) {
            if let Ok(content) = fs::read_to_string(path) {
                stats.processed_files += 1;
                
                let mut comment_count = 0;
                let cleaned_content = delete_file_content(&content, &comment_regex, &ignore_regex, doc_comment_prefix, &mut comment_count, path.to_str().unwrap_or(""), &mut stats);
                
                if comment_count > 0 {
                    stats.changed_files += 1;
                    stats.removed_comments += comment_count;
                    
                    if dry_run {
                        style::print_info(&format!("Would remove {} comments from {}", comment_count, path.display()));
                        print_comment_preview(&content, &cleaned_content, path.to_str().unwrap_or("file"));
                        
                        if let Some(base) = &output_base {
                            let file_name = path.file_name().unwrap();
                            let target_path = base.join(file_name);
                            
                            fs::write(&target_path, cleaned_content).map_err(|e| {
                                AppError::FileSystem { 
                                    path: target_path.clone(), 
                                    message: format!("Failed to write file: {}", e) 
                                }
                            })?;
                        }
                    } else {
                        let file_name = path.file_name().unwrap();
                        let target_path = match &output_base {
                            Some(base) => base.join(file_name),
                            None => path.to_path_buf(),
                        };
                        
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
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();
            
            if !file_path.is_file() || file_path.extension().and_then(|e| e.to_str()) != Some(file_extension) {
                continue;
            }
            
            if FileFilter::should_exclude(file_path) {
                continue;
            }
            
            if let Ok(content) = fs::read_to_string(file_path) {
                stats.processed_files += 1;
                
                let mut comment_count = 0;
                let cleaned_content = delete_file_content(&content, &comment_regex, &ignore_regex, doc_comment_prefix, &mut comment_count, file_path.to_str().unwrap_or(""), &mut stats);
                
                if comment_count > 0 {
                    stats.changed_files += 1;
                    stats.removed_comments += comment_count;
                    
                    if dry_run {
                        style::print_info(&format!("Would remove {} comments from {}", comment_count, file_path.display()));
                        print_comment_preview(&content, &cleaned_content, file_path.to_str().unwrap_or("file"));
                        
                        if let Some(base) = &output_base {
                            let rel_path = file_path.strip_prefix(path).unwrap_or(file_path);
                            let target = base.join(rel_path);
                            
                            if let Some(parent) = target.parent() {
                                fs::create_dir_all(parent).map_err(|e| {
                                    AppError::FileSystem { 
                                        path: parent.to_path_buf(), 
                                        message: format!("Failed to create directory: {}", e) 
                                    }
                                })?;
                            }
                            
                            fs::write(&target, cleaned_content).map_err(|e| {
                                AppError::FileSystem { 
                                    path: target.clone(), 
                                    message: format!("Failed to write file: {}", e) 
                                }
                            })?;
                        }
                    } else {
                        let target_path = match &output_base {
                            Some(base) => {
                                let rel_path = file_path.strip_prefix(path).unwrap_or(file_path);
                                let target = base.join(rel_path);
                                
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

fn delete_file_content(
    content: &str, 
    comment_regex: &Regex, 
    ignore_regex: &Regex, 
    doc_comment_prefix: &str,
    comment_count: &mut usize,
    file_path: &str,
    stats: &mut DeleteStats
) -> String {
    let mut result = String::with_capacity(content.len());
    
    let pattern = comment_regex.as_str();
    let is_python = pattern.starts_with("#");
    
    for line in content.lines() {
        let trimmed = line.trim_start();
        
        if line.trim().is_empty() {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if trimmed.starts_with(doc_comment_prefix) {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        
        if ignore_regex.is_match(line) {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        
        if line.contains('\\') {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        
        if line.contains("r#") {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        
        if !is_python && trimmed.starts_with("//") && !trimmed.starts_with("///") {
            *comment_count += 1;
            
            // Record the deleted comment for JSON output
            stats.deleted_comments.push(DeletedComment {
                file: file_path.to_string(),
                line: result.lines().count() + 1,
                comment_removed: line.to_string(),
            });
            
            continue;
        }
        
        if is_python && trimmed.starts_with("#") && !trimmed.starts_with("###") {
            *comment_count += 1;
            
            // Record the deleted comment for JSON output
            stats.deleted_comments.push(DeletedComment {
                file: file_path.to_string(),
                line: result.lines().count() + 1,
                comment_removed: line.to_string(),
            });
            
            continue;
        }
        
        if let Some((cleaned_line, removed_comment)) = process_line_preserving_strings(line, comment_regex, comment_count) {
            // Record the end-of-line comment that was removed
            if !removed_comment.is_empty() {
                stats.deleted_comments.push(DeletedComment {
                    file: file_path.to_string(),
                    line: result.lines().count() + 1,
                    comment_removed: removed_comment,
                });
            }
            
            result.push_str(&cleaned_line);
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    
    result
}

fn display_delete_results(stats: &DeleteStats, start_time: Instant) {
    let elapsed = start_time.elapsed();
    
    style::print_header("\n✅ Comment Deletion Complete");
    println!("📁 Files processed: {}", stats.processed_files);
    println!("🔄 Files changed: {}", stats.changed_files);
    println!("🗑️ Comments removed: {}", stats.removed_comments);
    
    if stats.changed_files == 0 {
        style::print_warning("⚠️ No files were changed - no comments found for deletion.");
        style::print_info("ℹ️ No PR will be created since there are no changes.");
    }
    
    style::print_success(&format!("⏱️ Deletion completed in {:.2?}", elapsed));
}

/// Process a line of code, preserving string literals while removing end-of-line comments
/// Returns Some((cleaned_line, removed_comment)) if a comment was found and removed, None if no comments were found
fn process_line_preserving_strings(line: &str, comment_regex: &Regex, comment_count: &mut usize) -> Option<(String, String)> {
    let mut in_string = false;
    let mut escape_next = false;
    let chars = line.chars().collect::<Vec<_>>();
    let length = chars.len();
    
    let mut comment_pos = None;
    
    let pattern = comment_regex.as_str();
    let is_python = pattern.starts_with("#");
    
    for i in 0..length {
        let c = chars[i];
        
        if escape_next {
            escape_next = false;
            continue;
        }
        
        match c {
            '\\' if in_string => {
                escape_next = true;
            },
            
            '"' => {
                in_string = !in_string;
            },
            
            '/' if !is_python && i + 1 < length && chars[i+1] == '/' && !in_string => {
                let prefix = &line[0..i];
                if !prefix.trim().is_empty() {
                    comment_pos = Some(i);
                    break;
                }
            },
            
            '#' if is_python && !in_string => {
                let prefix = &line[0..i];
                if !prefix.trim().is_empty() {
                    comment_pos = Some(i);
                    break;
                }
            },
            
            _ => {}
        }
    }
    
    if let Some(pos) = comment_pos {
        *comment_count += 1;
        let cleaned = line[0..pos].trim_end().to_string();
        let comment = line[pos..].trim_end().to_string();
        return Some((cleaned, comment));
    }
    
    None
}

fn print_comment_preview(original: &str, cleaned: &str, file_path: &str) {
    let original_lines: Vec<&str> = original.lines().collect();
    let cleaned_lines: Vec<&str> = cleaned.lines().collect();
    
    const MAX_PREVIEW_LINES: usize = 5;
    let mut preview_count = 0;
    
    println!("\n{}", style::bold(&format!("Preview of changes for {}", file_path)));
    
    let mut orig_pos = 0;
    let mut clean_pos = 0;
    
    while orig_pos < original_lines.len() && preview_count < MAX_PREVIEW_LINES {
        let original_line = original_lines[orig_pos];
        
        let trimmed = original_line.trim_start();
        if ((trimmed.starts_with("//") && !trimmed.starts_with("///")) || 
           (trimmed.starts_with("#") && !trimmed.starts_with("###"))) && 
           !original_line.contains("aicodeanalyzer: ignore") {
            println!("- {}", style::dimmed(original_line));
            orig_pos += 1;
            preview_count += 1;
            continue;
        }
        
        if clean_pos < cleaned_lines.len() {
            let cleaned_line = cleaned_lines[clean_pos];
            
            if original_line != cleaned_line && 
               (original_line.contains("//") || cleaned_line.contains("//") ||
                original_line.contains("#") || cleaned_line.contains("#")) {
                println!("- {}", style::dimmed(original_line));
                println!("+ {}", style::success(cleaned_line));
                orig_pos += 1;
                clean_pos += 1;
                preview_count += 1;
                continue;
            }
        }
        
        orig_pos += 1;
        clean_pos += 1;
    }
    
    let mut total_line_changes = 0;
    
    for line in original_lines.iter() {
        let trimmed = line.trim_start();
        if ((trimmed.starts_with("//") && !trimmed.starts_with("///")) || 
            (trimmed.starts_with("#") && !trimmed.starts_with("###"))) && 
            !line.contains("aicodeanalyzer: ignore") {
            total_line_changes += 1;
        }
    }
    
    for i in 0..std::cmp::min(original_lines.len(), cleaned_lines.len()) {
        if original_lines[i] != cleaned_lines[i] {
            total_line_changes += 1;
        }
    }
    
    if total_line_changes > MAX_PREVIEW_LINES {
        println!("... and {} more changes", total_line_changes - MAX_PREVIEW_LINES);
    }
    
    println!();
}

/// Export deleted comments as JSON output file
fn export_json_results(stats: &DeleteStats, base_dir: &str) -> AppResult<()> {
    // Skip export if no comments were removed
    if stats.deleted_comments.is_empty() {
        style::print_info("ℹ️ No comments to export to JSON");
        return Ok(());
    }
    
    let output_path = crate::output::path::resolve_output_path(
        "delete_comments",
        base_dir,
        "json"
    )?;
    
    // Create pretty-printed JSON
    let json_content = serde_json::to_string_pretty(&stats.deleted_comments)
        .map_err(|e| AppError::Analysis(format!("Failed to serialize JSON: {}", e)))?;
    
    // Write to file
    fs::write(&output_path, json_content).map_err(|e| AppError::FileSystem {
        path: output_path.clone(),
        message: format!("Failed to write JSON output: {}", e),
    })?;
    
    style::print_success(&format!(
        "📄 Exported {} deleted comments to JSON file: {}", 
        stats.deleted_comments.len(),
        output_path.display()
    ));
    
    Ok(())
}
