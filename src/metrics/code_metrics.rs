use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct CodeMetrics {
    pub total_files: usize,
    pub total_directories: usize,
    pub lines_of_code: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub by_language: HashMap<String, LanguageMetrics>,
}

#[derive(Debug, Default)]
pub struct LanguageMetrics {
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

    pub fn analyze_directory<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<(), String> {
        let path = dir_path.as_ref();
        
        if !path.exists() {
            return Err(format!("Path '{}' does not exist", path.display()));
        }
        
        if !path.is_dir() {
            return Err(format!("Path '{}' is not a directory", path.display()));
        }

        // Count directories and files while walking the directory
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            if path.is_dir() {
                self.total_directories += 1;
                continue;
            }
            
            self.total_files += 1;
            
            if let Some(extension) = path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    self.analyze_file(path, ext_str);
                }
            }
        }
        
        Ok(())
    }
    
    fn analyze_file<P: AsRef<Path>>(&mut self, file_path: P, extension: &str) {
        let language = match extension {
            "rs" => "Rust",
            "js" | "jsx" => "JavaScript",
            "ts" | "tsx" => "TypeScript",
            "py" => "Python",
            "java" => "Java",
            "c" | "h" => "C",
            "cpp" | "hpp" => "C++",
            "go" => "Go",
            "rb" => "Ruby",
            "php" => "PHP",
            "html" => "HTML",
            "css" => "CSS",
            "md" => "Markdown",
            "json" => "JSON",
            "yml" | "yaml" => "YAML",
            "toml" => "TOML",
            _ => "Other",
        };
        
        if let Ok(content) = fs::read_to_string(file_path) {
            let (loc, blank, comments) = count_lines(&content, language);
            
            self.lines_of_code += loc;
            self.blank_lines += blank;
            self.comment_lines += comments;
            
            let entry = self.by_language.entry(language.to_string()).or_insert_with(LanguageMetrics::default);
            entry.files += 1;
            entry.lines_of_code += loc;
            entry.blank_lines += blank;
            entry.comment_lines += comments;
        }
    }
    
    pub fn print_summary(&self) {
        println!("\nCode Metrics Summary:");
        println!("--------------------");
        println!("Total Directories: {}", self.total_directories);
        println!("Total Files: {}", self.total_files);
        println!("Total Lines of Code: {}", self.lines_of_code);
        println!("Total Blank Lines: {}", self.blank_lines);
        println!("Total Comment Lines: {}", self.comment_lines);
        
        if !self.by_language.is_empty() {
            println!("\nBreakdown by Language:");
            println!("---------------------");
            
            // Sort languages by lines of code for better readability
            let mut languages: Vec<(&String, &LanguageMetrics)> = self.by_language.iter().collect();
            languages.sort_by(|a, b| b.1.lines_of_code.cmp(&a.1.lines_of_code));
            
            for (language, metrics) in languages {
                println!("{}: {} files, {} lines of code", 
                    language, metrics.files, metrics.lines_of_code);
            }
        }
    }
}

fn count_lines(content: &str, language: &str) -> (usize, usize, usize) {
    let mut loc = 0;
    let mut blank = 0;
    let mut comments = 0;
    
    let (line_comment, block_comment_start, block_comment_end) = match language {
        "Rust" => ("//", "/*", "*/"),
        "JavaScript" | "TypeScript" | "C" | "C++" | "Java" | "Go" => ("//", "/*", "*/"),
        "Python" => ("#", "\"\"\"", "\"\"\""),
        "Ruby" => ("#", "=begin", "=end"),
        "HTML" | "CSS" => ("", "<!--", "-->"),
        "Markdown" | "YAML" | "TOML" | "JSON" => ("", "", ""),
        _ => ("", "", ""),
    };
    
    let mut in_block_comment = false;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            blank += 1;
            continue;
        }
        
        if in_block_comment {
            comments += 1;
            if !block_comment_end.is_empty() && trimmed.contains(block_comment_end) {
                in_block_comment = false;
            }
            continue;
        }
        
        if !block_comment_start.is_empty() && trimmed.contains(block_comment_start) {
            if !block_comment_end.is_empty() && trimmed.contains(block_comment_end) {
                comments += 1;
            } else {
                in_block_comment = true;
                comments += 1;
            }
            continue;
        }
        
        if !line_comment.is_empty() && trimmed.starts_with(line_comment) {
            comments += 1;
            continue;
        }
        
        loc += 1;
    }
    
    (loc, blank, comments)
}