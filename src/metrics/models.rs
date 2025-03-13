use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct CodeMetrics {
    // Overall totals
    pub total_files: usize,
    pub total_directories: usize,
    pub lines_of_code: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    
    // Production code metrics
    pub prod_files: usize,
    pub prod_lines_of_code: usize,
    pub prod_blank_lines: usize,
    pub prod_comment_lines: usize,
    
    // Test code metrics
    pub test_files: usize,
    pub test_lines_of_code: usize,
    pub test_blank_lines: usize,
    pub test_comment_lines: usize,
    
    // Metrics by language
    pub by_language: HashMap<String, LanguageMetrics>,
    
    // Separate maps for production and test code
    pub prod_by_language: HashMap<String, LanguageMetrics>,
    pub test_by_language: HashMap<String, LanguageMetrics>,
}

#[derive(Debug, Default, Clone)]
pub struct LanguageMetrics {
    pub language: String,
    pub files: usize,
    pub lines_of_code: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct FileMetrics {
    pub path: String,
    pub language: String,
    pub lines_of_code: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub is_test_file: bool,
}

impl CodeMetrics {
    pub fn new() -> Self {
        CodeMetrics {
            // Overall totals
            total_files: 0,
            total_directories: 0,
            lines_of_code: 0,
            blank_lines: 0,
            comment_lines: 0,
            
            // Production code metrics
            prod_files: 0,
            prod_lines_of_code: 0,
            prod_blank_lines: 0,
            prod_comment_lines: 0,
            
            // Test code metrics
            test_files: 0,
            test_lines_of_code: 0,
            test_blank_lines: 0,
            test_comment_lines: 0,
            
            // Metrics by language
            by_language: HashMap::new(),
            prod_by_language: HashMap::new(),
            test_by_language: HashMap::new(),
        }
    }

    pub fn add_language_metrics(&mut self, metrics: LanguageMetrics, file_path: &str) {
        // Update overall totals
        self.update_overall_metrics(&metrics);
        
        // Update language-specific metrics
        self.update_language_specific_metrics(&metrics);
        
        // Check if this is a test file and update corresponding metrics
        let is_test_file = self.is_test_file(file_path);
        
        if is_test_file {
            self.update_test_metrics(&metrics);
        } else {
            self.update_production_metrics(&metrics);
        }
    }
    
    fn update_overall_metrics(&mut self, metrics: &LanguageMetrics) {
        self.lines_of_code += metrics.lines_of_code;
        self.blank_lines += metrics.blank_lines;
        self.comment_lines += metrics.comment_lines;
    }
    
    fn update_language_specific_metrics(&mut self, metrics: &LanguageMetrics) {
        let entry = self
            .by_language
            .entry(metrics.language.clone())
            .or_insert_with(|| LanguageMetrics::new(metrics.language.clone()));

        entry.files += metrics.files;
        entry.lines_of_code += metrics.lines_of_code;
        entry.blank_lines += metrics.blank_lines;
        entry.comment_lines += metrics.comment_lines;
    }
    
    fn is_test_file(&self, file_path: &str) -> bool {
        file_path.contains("/test") || 
        file_path.contains("/tests/") || 
        file_path.contains("_test.") || 
        file_path.ends_with("_test.rs") ||
        file_path.ends_with("_tests.rs") ||
        file_path.ends_with("Test.java") ||
        file_path.ends_with(".test.js") ||
        file_path.ends_with(".test.ts") ||
        file_path.ends_with("_spec.js") ||
        file_path.ends_with("_spec.ts") ||
        file_path.ends_with("_test.py") ||
        file_path.ends_with("test_")
    }
    
    fn update_test_metrics(&mut self, metrics: &LanguageMetrics) {
        // Update test file counters
        self.test_files += metrics.files;
        self.test_lines_of_code += metrics.lines_of_code;
        self.test_blank_lines += metrics.blank_lines;
        self.test_comment_lines += metrics.comment_lines;
        
        // Update test language-specific metrics
        self.update_test_language_metrics(metrics);
    }
    
    fn update_test_language_metrics(&mut self, metrics: &LanguageMetrics) {
        let test_entry = self
            .test_by_language
            .entry(metrics.language.clone())
            .or_insert_with(|| LanguageMetrics::new(metrics.language.clone()));
            
        test_entry.files += metrics.files;
        test_entry.lines_of_code += metrics.lines_of_code;
        test_entry.blank_lines += metrics.blank_lines;
        test_entry.comment_lines += metrics.comment_lines;
    }
    
    fn update_production_metrics(&mut self, metrics: &LanguageMetrics) {
        // Update production file counters
        self.prod_files += metrics.files;
        self.prod_lines_of_code += metrics.lines_of_code;
        self.prod_blank_lines += metrics.blank_lines;
        self.prod_comment_lines += metrics.comment_lines;
        
        // Update production language-specific metrics
        self.update_production_language_metrics(metrics);
    }
    
    fn update_production_language_metrics(&mut self, metrics: &LanguageMetrics) {
        let prod_entry = self
            .prod_by_language
            .entry(metrics.language.clone())
            .or_insert_with(|| LanguageMetrics::new(metrics.language.clone()));
            
        prod_entry.files += metrics.files;
        prod_entry.lines_of_code += metrics.lines_of_code;
        prod_entry.blank_lines += metrics.blank_lines;
        prod_entry.comment_lines += metrics.comment_lines;
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
