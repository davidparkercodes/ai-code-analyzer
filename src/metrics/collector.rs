use crate::analyzer::file_analyzer::FileAnalyzer;
use crate::metrics::models::CodeMetrics;
use ignore::WalkBuilder;
use std::path::Path;

pub struct MetricsCollector {
    file_analyzer: FileAnalyzer,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
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

        for result in walker {
            let entry = match result {
                Ok(entry) => entry,
                Err(err) => {
                    // Skip entries we can't access with a warning in debug
                    crate::output::style::print_warning(&format!("Warning: {}", err));
                    continue;
                }
            };

            let path = entry.path();

            if path.is_dir() {
                metrics.total_directories += 1;
                continue;
            }

            metrics.total_files += 1;

            if let Some(file_metrics) = self.file_analyzer.analyze_file(path) {
                metrics.add_language_metrics(file_metrics, &path.to_string_lossy());
            }
        }

        Ok(metrics)
    }
}
