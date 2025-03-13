use crate::analyzer::file_analyzer::FileAnalyzer;
use crate::cache::AnalysisCache;
use crate::metrics::models::CodeMetrics;
use ignore::{DirEntry, WalkBuilder};
use rayon::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct MetricsCollector {
    file_analyzer: FileAnalyzer,
    cache: Arc<AnalysisCache>,
    parallel: bool,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

use crate::util::parallel::ParallelProcessing;

impl MetricsCollector {
    pub fn new() -> Self {
        let cache = Arc::new(AnalysisCache::new());
        MetricsCollector {
            file_analyzer: FileAnalyzer::with_cache(Arc::clone(&cache)),
            cache,
            parallel: true,
        }
    }
}

impl ParallelProcessing for MetricsCollector {
    fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }
    
    fn is_parallel(&self) -> bool {
        self.parallel
    }
}

impl MetricsCollector {
    pub fn collect_metrics<P: AsRef<Path>>(&self, dir_path: P) -> Result<CodeMetrics, String> {
        let path = dir_path.as_ref();

        if !path.exists() {
            return Err(format!("Path '{}' does not exist", path.display()));
        }

        if !path.is_dir() {
            return Err(format!("Path '{}' is not a directory", path.display()));
        }

        let metrics = Arc::new(Mutex::new(CodeMetrics::new()));
        let dir_count = Arc::new(Mutex::new(0));
        
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
                file_name != ".DS_Store"
            })
            .build();
        
        // Collect all entries first to enable parallel processing
        let entries: Vec<DirEntry> = walker
            .filter_map(|result| {
                match result {
                    Ok(entry) => Some(entry),
                    Err(err) => {
                        crate::output::style::print_warning(&format!("Warning: {}", err));
                        None
                    }
                }
            })
            .collect();
            
        // Process directory entries to count total directories
        for entry in &entries {
            if entry.path().is_dir() {
                let mut count = dir_count.lock().unwrap();
                *count += 1;
                continue;
            }
        }
        
        // Filter to get only file entries
        let file_entries: Vec<&DirEntry> = entries
            .iter()
            .filter(|e| !e.path().is_dir())
            .collect();
            
        let process_entry = |entry: &DirEntry| {
            let path = entry.path();
            
            if let Some(file_metrics) = self.file_analyzer.analyze_file(path) {
                let mut metrics_guard = metrics.lock().unwrap();
                metrics_guard.total_files += 1;
                metrics_guard.add_language_metrics(file_metrics, &path.to_string_lossy());
            }
        };
        
        if self.parallel {
            // Process files in parallel
            file_entries.par_iter().for_each(|entry| {
                process_entry(entry);
            });
        } else {
            // Process files sequentially
            file_entries.iter().for_each(|entry| {
                process_entry(entry);
            });
        }
        
        // Update the directory count
        let mut metrics_result = metrics.lock().unwrap();
        metrics_result.total_directories = *dir_count.lock().unwrap();
        
        // Purge any stale entries from the cache periodically
        self.cache.purge_stale_entries();
        
        Ok((*metrics_result).clone())
    }
}