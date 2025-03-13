use crate::analyzer::file_analyzer::FileAnalyzer;
use crate::cache::AnalysisCache;
use crate::metrics::models::CodeMetrics;
use crate::util::file_filter::FileFilter;
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

impl MetricsCollector {
    pub fn collect_metrics<P: AsRef<Path>>(&self, dir_path: P) -> Result<CodeMetrics, String> {
        let path = dir_path.as_ref();
        self.validate_directory_path(path)?;
        
        let metrics = Arc::new(Mutex::new(CodeMetrics::new()));
        let dir_count = Arc::new(Mutex::new(0));
        
        let entries = self.walk_directory(path);
        self.count_directories(&entries, &dir_count);
        let file_entries = self.filter_file_entries(&entries);
        
        self.process_file_entries(&file_entries, &metrics);
        
        let metrics_result = self.finalize_metrics(&metrics, &dir_count);
        
        // Purge any stale entries from the cache periodically
        self.cache.purge_stale_entries();
        
        Ok(metrics_result)
    }
    
    fn validate_directory_path<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(format!("Path '{}' does not exist", path.display()));
        }

        if !path.is_dir() {
            return Err(format!("Path '{}' is not a directory", path.display()));
        }
        
        Ok(())
    }
    
    fn walk_directory<P: AsRef<Path>>(&self, path: P) -> Vec<DirEntry> {
        let walker = WalkBuilder::new(path)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .filter_entry(|e| {
                !FileFilter::should_exclude(e.path())
            })
            .build();
            
        walker
            .filter_map(|result| {
                match result {
                    Ok(entry) => Some(entry),
                    Err(error) => {
                        crate::output::style::print_warning(&format!("Warning: {}", error));
                        None
                    }
                }
            })
            .collect()
    }
    
    fn count_directories(&self, entries: &[DirEntry], dir_count: &Arc<Mutex<usize>>) {
        for entry in entries {
            if entry.path().is_dir() {
                let mut count = dir_count.lock().unwrap();
                *count += 1;
            }
        }
    }
    
    fn filter_file_entries<'a>(&self, entries: &'a [DirEntry]) -> Vec<&'a DirEntry> {
        entries
            .iter()
            .filter(|e| !e.path().is_dir())
            .collect()
    }
    
    fn process_file_entries(&self, file_entries: &[&DirEntry], metrics: &Arc<Mutex<CodeMetrics>>) {
        let process_entry = |entry: &DirEntry| {
            let path = entry.path();
            
            if let Some(file_metrics) = self.file_analyzer.analyze_file(path) {
                let mut metrics_guard = metrics.lock().unwrap();
                metrics_guard.total_files += 1;
                metrics_guard.add_language_metrics(file_metrics, &path.to_string_lossy());
            }
        };
        
        if self.parallel {
            file_entries.par_iter().for_each(|entry| {
                process_entry(entry);
            });
        } else {
            file_entries.iter().for_each(|entry| {
                process_entry(entry);
            });
        }
    }
    
    fn finalize_metrics(&self, metrics: &Arc<Mutex<CodeMetrics>>, dir_count: &Arc<Mutex<usize>>) -> CodeMetrics {
        let mut metrics_result = metrics.lock().unwrap();
        metrics_result.total_directories = *dir_count.lock().unwrap();
        
        (*metrics_result).clone()
    }
}