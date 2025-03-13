use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CodeMetrics {
    pub total_files: usize,
    pub total_directories: usize,
    pub lines_of_code: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub by_language: HashMap<String, LanguageMetrics>,
}

#[derive(Debug, Default, Clone)]
pub struct LanguageMetrics {
    pub language: String,
    pub files: usize,
    pub lines_of_code: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
}

impl CodeMetrics {
    pub fn new() -> Self {
        CodeMetrics {
            total_files: 0,
            total_directories: 0,
            lines_of_code: 0,
            blank_lines: 0,
            comment_lines: 0,
            by_language: HashMap::new(),
        }
    }

    pub fn add_language_metrics(&mut self, metrics: LanguageMetrics) {
        self.lines_of_code += metrics.lines_of_code;
        self.blank_lines += metrics.blank_lines;
        self.comment_lines += metrics.comment_lines;

        let entry = self
            .by_language
            .entry(metrics.language.clone())
            .or_insert_with(|| LanguageMetrics::new(metrics.language.clone()));

        entry.files += metrics.files;
        entry.lines_of_code += metrics.lines_of_code;
        entry.blank_lines += metrics.blank_lines;
        entry.comment_lines += metrics.comment_lines;
    }
}

impl LanguageMetrics {
    pub fn new(language: String) -> Self {
        LanguageMetrics {
            language,
            files: 0,
            lines_of_code: 0,
            blank_lines: 0,
            comment_lines: 0,
        }
    }
}
