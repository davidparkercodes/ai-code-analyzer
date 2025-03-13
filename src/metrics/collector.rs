use crate::analyzer::file_analyzer::FileAnalyzer;
use crate::metrics::models::CodeMetrics;
use ignore::WalkBuilder;
use std::path::Path;

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
        
        // Build a walker that respects .gitignore
        let walker = WalkBuilder::new(path)
            .hidden(false) // Process hidden files, but still respect .gitignore
            .git_ignore(true) // Use .gitignore files
            .git_global(true) // Use global gitignore
            .git_exclude(true) // Use git exclude files
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
                metrics.add_language_metrics(file_metrics);
            }
        }

        Ok(metrics)
    }
}
