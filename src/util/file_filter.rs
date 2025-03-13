use std::path::Path;

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
        
        path_str.contains("/test") || 
        path_str.contains("/tests/") || 
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
        Self::is_system_file(&path) || Self::is_binary_or_media_file(&path)
    }
}