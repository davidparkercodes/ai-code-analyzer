use std::path::Path;
use walkdir::WalkDir;
use crate::analyzer::file_analyzer::FileAnalyzer;
use crate::metrics::models::CodeMetrics;

pub struct MetricsCollector {
    file_analyzer: FileAnalyzer,
}

impl MetricsCollector {
    pub fn new() -> Self {
        MetricsCollector {
            file_analyzer: FileAnalyzer::new(),
        }
    }
    
    pub fn collect_metrics<P: AsRef<Path>>(&self, dir_path: P) -> Result<CodeMetrics, String> {
        let path = dir_path.as_ref();
        
        if !path.exists() {
            return Err(format!("Path '{}' does not exist", path.display()));
        }
        
        if !path.is_dir() {
            return Err(format!("Path '{}' is not a directory", path.display()));
        }
        
        let mut metrics = CodeMetrics::new();
        
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            if path.is_dir() {
                metrics.total_directories += 1;
                continue;
            }
            
            metrics.total_files += 1;
            
            if let Some(file_metrics) = self.file_analyzer.analyze_file(path) {
                metrics.add_language_metrics(file_metrics);
            }
        }
        
        Ok(metrics)
    }
}