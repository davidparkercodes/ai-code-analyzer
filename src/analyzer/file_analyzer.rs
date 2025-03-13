use crate::cache::AnalysisCache;
use crate::metrics::language::LanguageDetector;
use crate::metrics::models::{FileMetrics, LanguageMetrics};
use std::fs;
use std::path::Path;
use std::sync::Arc;

pub struct FileAnalyzer {
    language_detector: LanguageDetector,
    cache: Arc<AnalysisCache>,
}

impl Default for FileAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl FileAnalyzer {
    pub fn new() -> Self {
        FileAnalyzer {
            language_detector: LanguageDetector::new(),
            cache: Arc::new(AnalysisCache::new()),
        }
    }
    
    pub fn with_cache(cache: Arc<AnalysisCache>) -> Self {
        FileAnalyzer {
            language_detector: LanguageDetector::new(),
            cache,
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&self, file_path: P) -> Option<LanguageMetrics> {
        let path = file_path.as_ref();
        let path_str = path.to_string_lossy().to_string();
        
        // Check if we have cached metrics for this file
        if let Some(file_metrics) = self.cache.get_metrics(&path_str) {
            let mut metrics = LanguageMetrics::new(file_metrics.language);
            metrics.files = 1;
            metrics.lines_of_code = file_metrics.lines_of_code;
            metrics.blank_lines = file_metrics.blank_lines;
            metrics.comment_lines = file_metrics.comment_lines;
            return Some(metrics);
        }
        
        let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        
        // Try to get language from cache first
        let language = if let Some(cached_lang) = self.cache.get_language(&path_str) {
            cached_lang
        } else {
            let detected_lang = if extension.is_empty() && !file_name.is_empty() {
                self.language_detector.detect_by_filename(file_name)
            } else {
                self.language_detector.detect_language(extension)
            };
            // Cache the detected language
            self.cache.cache_language(&path_str, detected_lang.clone());
            detected_lang
        };

        // Try to get content from cache first
        let content = if let Some(cached_content) = self.cache.get_file_content(&path_str) {
            cached_content
        } else if let Ok(file_content) = fs::read_to_string(path) {
            // Cache the file content
            self.cache.cache_file_content(&path_str, file_content.clone());
            file_content
        } else {
            return None;
        };

        let (loc, blank, comments) = self.count_lines(&content, &language);

        let mut metrics = LanguageMetrics::new(language.clone());
        metrics.files = 1;
        metrics.lines_of_code = loc;
        metrics.blank_lines = blank;
        metrics.comment_lines = comments;

        // Check if this is a test file
        let is_test_file = path_str.contains("/test") || 
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
            path_str.ends_with("test_");

        // Cache the file metrics
        let file_metrics = FileMetrics {
            path: path_str.clone(),
            language,
            lines_of_code: loc,
            blank_lines: blank,
            comment_lines: comments,
            is_test_file,
        };
        self.cache.cache_metrics(&path_str, file_metrics);

        Some(metrics)
    }

    fn count_lines(&self, content: &str, language: &str) -> (usize, usize, usize) {
        let mut loc = 0;
        let mut blank = 0;
        let mut comments = 0;

        let (line_comment, block_comment_start, block_comment_end) =
            self.language_detector.get_comment_syntax(language);

        let mut in_block_comment = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                blank += 1;
                continue;
            }

            if in_block_comment {
                comments += 1;
                if !block_comment_end.is_empty() && trimmed.contains(&block_comment_end) {
                    in_block_comment = false;
                }
                continue;
            }

            if !block_comment_start.is_empty() && trimmed.contains(&block_comment_start) {
                if !block_comment_end.is_empty() && trimmed.contains(&block_comment_end) {
                    comments += 1;
                } else {
                    in_block_comment = true;
                    comments += 1;
                }
                continue;
            }

            if !line_comment.is_empty() && trimmed.starts_with(&line_comment) {
                comments += 1;
                continue;
            }

            loc += 1;
        }

        (loc, blank, comments)
    }
}
