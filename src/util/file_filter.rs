use std::path::{Path, PathBuf};
use std::fs;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

/// Common file filtering utilities for determining which files to include in analysis
pub struct FileFilter;

impl FileFilter {
    /// Checks if a file is a system file that should be excluded from analysis
    pub fn is_system_file<P: AsRef<Path>>(path: P) -> bool {
        let path_str = path.as_ref().to_string_lossy();
        let file_name = path.as_ref().file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
            
        path_str.contains("/.git/") || 
        path_str.ends_with(".lock") || 
        path_str.ends_with(".gitignore") ||
        file_name == ".DS_Store"
    }
    
    /// Checks if a file is a test file that should be analyzed separately
    pub fn is_test_file<P: AsRef<Path>>(path: P) -> bool {
        let path_str = path.as_ref().to_string_lossy();
        
        path_str.contains("/test/") || 
        path_str.contains("/tests/") || 
        path_str.ends_with("/test") ||
        path_str.ends_with("/tests") ||
        path_str.contains("_test.") || 
        path_str.ends_with("_test.rs") ||
        path_str.ends_with("_tests.rs") ||
        path_str.ends_with("Test.java") ||
        path_str.ends_with(".test.js") ||
        path_str.ends_with(".test.ts") ||
        path_str.ends_with("_spec.js") ||
        path_str.ends_with("_spec.ts") ||
        path_str.ends_with("_test.py") ||
        path_str.ends_with("test_") ||
        path_str.contains("__tests__") ||
        path_str.contains("__test__")
    }
    
    /// Checks if a file is a binary or media file that should be excluded
    pub fn is_binary_or_media_file<P: AsRef<Path>>(path: P) -> bool {
        let path_str = path.as_ref().to_string_lossy();
        
        path_str.ends_with(".png") ||
        path_str.ends_with(".jpg") ||
        path_str.ends_with(".jpeg") ||
        path_str.ends_with(".gif") ||
        path_str.ends_with(".svg") ||
        path_str.ends_with(".woff") ||
        path_str.ends_with(".woff2") ||
        path_str.ends_with(".ttf") ||
        path_str.ends_with(".eot") ||
        path_str.ends_with(".ico") ||
        path_str.ends_with(".pdf") ||
        path_str.ends_with(".zip") ||
        path_str.ends_with(".tar") ||
        path_str.ends_with(".gz") ||
        path_str.ends_with(".exe") ||
        path_str.ends_with(".bin")
    }
    
    /// Checks if a file should be excluded from analysis for any reason
    pub fn should_exclude<P: AsRef<Path>>(path: P) -> bool {
        Self::is_system_file(&path) || Self::is_binary_or_media_file(&path) || Self::is_test_file(&path)
    }
    
    /// Check if a file is a source code file that should be included in analysis
    pub fn is_source_file<P: AsRef<Path>>(path: P) -> bool {
        if Self::should_exclude(&path) {
            return false;
        }
        
        let path_ref = path.as_ref();
        if !path_ref.is_file() {
            return false;
        }
        
        let extension = path_ref.extension().and_then(|e| e.to_str()).unwrap_or("");
        
        !extension.is_empty()
    }
}

/// Get all source files in a directory recursively
pub fn get_all_source_files(path: &str, parallel: bool) -> std::io::Result<Vec<PathBuf>> {
    let root_path = Path::new(path);
    if !root_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Path not found: {}", path)
        ));
    }
    
    if root_path.is_file() {
        if FileFilter::is_source_file(root_path) {
            return Ok(vec![root_path.to_path_buf()]);
        } else {
            return Ok(vec![]);
        }
    }
    
    if parallel {
        get_source_files_parallel(root_path)
    } else {
        get_source_files_sequential(root_path)
    }
}

fn get_source_files_sequential(root_path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut source_files = Vec::new();
    visit_dirs(root_path, &mut source_files)?;
    Ok(source_files)
}

fn visit_dirs(dir: &Path, source_files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if dir_name == "node_modules" || dir_name == "target" || dir_name == ".git" {
                    continue;
                }
                visit_dirs(&path, source_files)?;
            } else if FileFilter::is_source_file(&path) {
                source_files.push(path);
            }
        }
    }
    Ok(())
}

fn get_source_files_parallel(root_path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let source_files = Arc::new(Mutex::new(Vec::new()));
    
    let mut dirs_to_process = Vec::new();
    dirs_to_process.push(root_path.to_path_buf());
    
    for entry in walkdir::WalkDir::new(root_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| {
            let is_dir = e.path().is_dir();
            let path_str = e.path().to_string_lossy();
            
            is_dir && !path_str.contains("node_modules") && 
            !path_str.contains("/target") && 
            !path_str.contains("/.git")
        })
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
    {
        dirs_to_process.push(entry.path().to_path_buf());
    }
    
    dirs_to_process.into_par_iter().for_each(|dir| {
        let mut local_files = Vec::new();
        if let Err(_) = visit_dirs(&dir, &mut local_files) {
            return;
        }
        
        let mut files = source_files.lock().unwrap();
        files.extend(local_files);
    });
    
    let result = source_files.lock().unwrap().clone();
    Ok(result)
}
