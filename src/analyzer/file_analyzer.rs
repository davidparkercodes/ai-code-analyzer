use crate::metrics::language::LanguageDetector;
use crate::metrics::models::LanguageMetrics;
use std::fs;
use std::path::Path;

pub struct FileAnalyzer {
    language_detector: LanguageDetector,
}

impl FileAnalyzer {
    pub fn new() -> Self {
        FileAnalyzer {
            language_detector: LanguageDetector::new(),
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&self, file_path: P) -> Option<LanguageMetrics> {
        let path = file_path.as_ref();

        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        let language = self.language_detector.detect_language(extension);

        if let Ok(content) = fs::read_to_string(path) {
            let (loc, blank, comments) = self.count_lines(&content, &language);

            let mut metrics = LanguageMetrics::new(language);
            metrics.files = 1;
            metrics.lines_of_code = loc;
            metrics.blank_lines = blank;
            metrics.comment_lines = comments;

            return Some(metrics);
        }

        None
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
