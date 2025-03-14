use code_analyzer::cache::AnalysisCache;
use code_analyzer::metrics::models::FileMetrics;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_file_content_cache() {
    let cache = AnalysisCache::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    
    let content = "This is test content";
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }
    
    let path_str = file_path.to_string_lossy().to_string();
    
    assert_eq!(cache.get_file_content(&path_str), None);
    
    cache.cache_file_content(&path_str, content.to_string());
    
    assert_eq!(cache.get_file_content(&path_str), Some(content.to_string()));
}

#[test]
fn test_language_cache() {
    let cache = AnalysisCache::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"fn main() {}").unwrap();
    }
    
    let path_str = file_path.to_string_lossy().to_string();
    
    assert_eq!(cache.get_language(&path_str), None);
    
    cache.cache_language(&path_str, "Rust".to_string());
    
    assert_eq!(cache.get_language(&path_str), Some("Rust".to_string()));
}

#[test]
fn test_metrics_cache() {
    let cache = AnalysisCache::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"fn main() {\n    println!(\"Hello\");\n}").unwrap();
    }
    
    let path_str = file_path.to_string_lossy().to_string();
    
    assert_eq!(cache.get_metrics(&path_str), None);
    
    let metrics = FileMetrics {
        path: path_str.clone(),
        language: "Rust".to_string(),
        lines_of_code: 3,
        blank_lines: 0,
        comment_lines: 0,
        is_test_file: false,
    };
    
    cache.cache_metrics(&path_str, metrics.clone());
    
    let cached_metrics = cache.get_metrics(&path_str).unwrap();
    assert_eq!(cached_metrics.language, "Rust");
    assert_eq!(cached_metrics.lines_of_code, 3);
    assert_eq!(cached_metrics.blank_lines, 0);
    assert_eq!(cached_metrics.comment_lines, 0);
}

#[test]
fn test_dependencies_cache() {
    let cache = AnalysisCache::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"use std::io;\nuse std::path;").unwrap();
    }
    
    let path_str = file_path.to_string_lossy().to_string();
    
    assert_eq!(cache.get_dependencies(&path_str), None);
    
    let deps = vec!["std::io".to_string(), "std::path".to_string()];
    cache.cache_dependencies(&path_str, deps.clone());
    
    let cached_deps = cache.get_dependencies(&path_str).unwrap();
    assert_eq!(cached_deps, deps);
}

#[test]
fn test_purge_stale_entries() {
    let cache = AnalysisCache::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"fn main() {}").unwrap();
    }
    
    let path_str = file_path.to_string_lossy().to_string();
    let non_existent_path = dir.path().join("non_existent.rs").to_string_lossy().to_string();
    
    cache.cache_language(&path_str, "Rust".to_string());
    cache.cache_language(&non_existent_path, "Rust".to_string());
    
    cache.purge_stale_entries();
    
    assert_eq!(cache.get_language(&path_str), Some("Rust".to_string()));
    
    assert_eq!(cache.get_language(&non_existent_path), None);
}
