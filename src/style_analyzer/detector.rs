// FileAnalyzer import removed as it's not used
use crate::metrics::language::LanguageDetector;
use crate::style_analyzer::pattern::{
    BracketStyle, IndentationStyle, NamingConvention, StylePattern, StylePatternCollection, StyleRule,
};
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

use super::report::StyleReport;

pub struct StyleDetector {
    language_detector: LanguageDetector,
}

impl StyleDetector {
    pub fn new() -> Self {
        StyleDetector {
            language_detector: LanguageDetector::new(),
        }
    }

    pub fn detect_patterns<P: AsRef<Path>>(&self, dir_path: P) -> Result<StyleReport, String> {
        let path = dir_path.as_ref();

        if !path.exists() {
            return Err(format!("Path '{}' does not exist", path.display()));
        }

        if !path.is_dir() {
            return Err(format!("Path '{}' is not a directory", path.display()));
        }

        let patterns = Arc::new(Mutex::new(StylePatternCollection::new()));
        let language_files = Arc::new(Mutex::new(HashMap::<String, usize>::new()));

        // Create walker to traverse the directory
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

        // Collect all file entries
        let entries: Vec<_> = walker
            .filter_map(|result| {
                match result {
                    Ok(entry) if !entry.path().is_dir() => Some(entry),
                    _ => None,
                }
            })
            .collect();

        // Process files in parallel
        entries.par_iter().for_each(|entry| {
            let file_path = entry.path();
            
            // Detect language
            let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            let file_name = file_path.file_name().and_then(|name| name.to_str()).unwrap_or("");
            
            let language = if extension.is_empty() && !file_name.is_empty() {
                self.language_detector.detect_by_filename(file_name)
            } else {
                self.language_detector.detect_language(extension)
            };
            
            // Count language files
            {
                let mut lang_files = language_files.lock().unwrap();
                *lang_files.entry(language.clone()).or_insert(0) += 1;
            }
            
            // Read file content
            if let Ok(content) = fs::read_to_string(file_path) {
                self.analyze_file_style(&content, &language, file_path.to_string_lossy().as_ref(), patterns.clone());
            }
        });

        // Create the style report
        let pattern_collection = patterns.lock().unwrap();
        let languages_map = language_files.lock().unwrap();
        
        let mut report = StyleReport::new();
        
        // Process each language
        for (language, file_count) in languages_map.iter() {
            report.add_language_stats(language, *file_count);
            
            // Add patterns for this language
            for pattern in pattern_collection.get_patterns(language) {
                report.add_pattern(pattern.clone());
            }
        }
        
        // Generate dominant style guide
        report.generate_style_guide();
        
        Ok(report)
    }

    fn analyze_file_style(&self, content: &str, language: &str, file_path: &str, patterns: Arc<Mutex<StylePatternCollection>>) {
        // Detect indentation style
        self.detect_indentation_style(content, language, patterns.clone());
        
        // Detect bracket style
        self.detect_bracket_style(content, language, patterns.clone());
        
        // Detect naming conventions
        self.detect_naming_conventions(content, language, file_path, patterns.clone());
        
        // Detect line length
        self.detect_line_length(content, language, patterns);
    }

    fn detect_indentation_style(&self, content: &str, language: &str, patterns: Arc<Mutex<StylePatternCollection>>) {
        let mut space_indent_counts = HashMap::new();
        let mut tab_count = 0;
        
        let lines: Vec<&str> = content.lines().collect();
        let mut total_indent_lines = 0;
        
        for i in 0..lines.len() {
            let line = lines[i];
            if line.trim().is_empty() {
                continue;
            }
            
            // Count leading whitespace
            let mut spaces = 0;
            let mut tabs = 0;
            
            for c in line.chars() {
                match c {
                    ' ' => spaces += 1,
                    '\t' => tabs += 1,
                    _ => break,
                }
            }
            
            if spaces > 0 || tabs > 0 {
                total_indent_lines += 1;
                
                if tabs > 0 {
                    tab_count += 1;
                } else if spaces > 0 {
                    *space_indent_counts.entry(spaces).or_insert(0) += 1;
                }
            }
        }
        
        if total_indent_lines > 0 {
            let style_rule = if tab_count > 0 && space_indent_counts.values().sum::<usize>() > 0 {
                // Mixed tabs and spaces
                StyleRule::IndentationStyle(IndentationStyle::Mixed)
            } else if tab_count > 0 {
                // Tabs only
                StyleRule::IndentationStyle(IndentationStyle::Tabs)
            } else {
                // Find most common space count
                let mut max_count = 0;
                let mut most_common_space = 0;
                
                for (&space, &count) in &space_indent_counts {
                    if count > max_count {
                        max_count = count;
                        most_common_space = space;
                    }
                }
                
                StyleRule::IndentationStyle(IndentationStyle::Spaces(most_common_space))
            };
            
            let mut patterns_lock = patterns.lock().unwrap();
            let mut pattern = StylePattern::new(style_rule, language);
            pattern.add_occurrence(None);
            
            let example = lines.iter()
                .filter(|line| !line.trim().is_empty())
                .take(3)
                .map(|&s| s.to_string())
                .collect::<Vec<String>>()
                .join("\n");
                
            pattern.examples.push(example);
            pattern.update_consistency(total_indent_lines);
            
            patterns_lock.add_pattern(pattern);
        }
    }

    fn detect_bracket_style(&self, content: &str, language: &str, patterns: Arc<Mutex<StylePatternCollection>>) {
        // Simple heuristic for bracket style detection
        let mut same_line_count = 0;
        let mut new_line_count = 0;
        
        let lines: Vec<&str> = content.lines().collect();
        
        for i in 0..lines.len().saturating_sub(1) {
            let line = lines[i];
            let next_line = lines[i + 1];
            
            // Check patterns like "function() {" vs "function()\n{"
            if line.contains("(") && line.contains(")") {
                if line.trim().ends_with("{") {
                    same_line_count += 1;
                } else if next_line.trim() == "{" {
                    new_line_count += 1;
                }
            }
        }
        
        let total_bracket_lines = same_line_count + new_line_count;
        
        if total_bracket_lines > 0 {
            let style_rule = if same_line_count > 0 && new_line_count > 0 {
                StyleRule::BracketStyle(BracketStyle::Mixed)
            } else if same_line_count > 0 {
                StyleRule::BracketStyle(BracketStyle::SameLine)
            } else {
                StyleRule::BracketStyle(BracketStyle::NewLine)
            };
            
            let mut patterns_lock = patterns.lock().unwrap();
            let mut pattern = StylePattern::new(style_rule, language);
            pattern.add_occurrence(None);
            
            // Find examples
            for i in 0..lines.len().saturating_sub(2) {
                if pattern.examples.len() >= 3 {
                    break;
                }
                
                let current = lines[i];
                let next = lines[i + 1];
                
                if current.contains("(") && current.contains(")") {
                    if current.trim().ends_with("{") && same_line_count > new_line_count {
                        pattern.examples.push(current.to_string());
                    } else if next.trim() == "{" && new_line_count > same_line_count {
                        pattern.examples.push(format!("{}\n{}", current, next));
                    }
                }
            }
            
            pattern.update_consistency(total_bracket_lines);
            patterns_lock.add_pattern(pattern);
        }
    }

    fn detect_naming_conventions(&self, content: &str, language: &str, file_path: &str, patterns: Arc<Mutex<StylePatternCollection>>) {
        // Simplified detection for common identifiers
        let re_camel_case = regex::Regex::new(r"\b[a-z][a-zA-Z0-9]*[A-Z][a-zA-Z0-9]*\b").unwrap();
        let re_pascal_case = regex::Regex::new(r"\b[A-Z][a-zA-Z0-9]*\b").unwrap();
        let re_snake_case = regex::Regex::new(r"\b[a-z][a-z0-9]*(_[a-z0-9]+)+\b").unwrap();
        let re_screaming_snake = regex::Regex::new(r"\b[A-Z][A-Z0-9]*(_[A-Z0-9]+)+\b").unwrap();
        
        let camel_count = re_camel_case.find_iter(content).count();
        let pascal_count = re_pascal_case.find_iter(content).count();
        let snake_count = re_snake_case.find_iter(content).count();
        let screaming_count = re_screaming_snake.find_iter(content).count();
        
        let total_identifiers = camel_count + pascal_count + snake_count + screaming_count;
        
        if total_identifiers > 0 {
            // Find dominant naming convention
            let convention = if camel_count > pascal_count && camel_count > snake_count && camel_count > screaming_count {
                NamingConvention::CamelCase
            } else if pascal_count > camel_count && pascal_count > snake_count && pascal_count > screaming_count {
                NamingConvention::PascalCase
            } else if snake_count > camel_count && snake_count > pascal_count && snake_count > screaming_count {
                NamingConvention::SnakeCase
            } else if screaming_count > camel_count && screaming_count > pascal_count && screaming_count > snake_count {
                NamingConvention::ScreamingSnakeCase
            } else {
                NamingConvention::Mixed
            };
            
            // If multiple conventions with significant presence, mark as mixed
            let threshold = total_identifiers as f64 * 0.25; // 25% threshold
            let mut mixed = false;
            
            if (camel_count as f64 > threshold && convention != NamingConvention::CamelCase) ||
               (pascal_count as f64 > threshold && convention != NamingConvention::PascalCase) ||
               (snake_count as f64 > threshold && convention != NamingConvention::SnakeCase) ||
               (screaming_count as f64 > threshold && convention != NamingConvention::ScreamingSnakeCase) {
                mixed = true;
            }
            
            let style_rule = StyleRule::NamingConvention(
                if mixed { NamingConvention::Mixed } else { convention }
            );
            
            let mut patterns_lock = patterns.lock().unwrap();
            let mut pattern = StylePattern::new(style_rule, language);
            pattern.add_occurrence(Some(file_path.to_string()));
            
            // Get examples for each naming convention
            if camel_count > 0 {
                if let Some(m) = re_camel_case.find(content) {
                    pattern.examples.push(format!("camelCase: {}", m.as_str()));
                }
            }
            
            if pascal_count > 0 && pattern.examples.len() < 3 {
                if let Some(m) = re_pascal_case.find(content) {
                    pattern.examples.push(format!("PascalCase: {}", m.as_str()));
                }
            }
            
            if snake_count > 0 && pattern.examples.len() < 3 {
                if let Some(m) = re_snake_case.find(content) {
                    pattern.examples.push(format!("snake_case: {}", m.as_str()));
                }
            }
            
            pattern.update_consistency(total_identifiers);
            patterns_lock.add_pattern(pattern);
        }
    }

    fn detect_line_length(&self, content: &str, language: &str, patterns: Arc<Mutex<StylePatternCollection>>) {
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return;
        }
        
        // Collect line lengths
        let mut length_counts = HashMap::new();
        let mut total_lines = 0;
        
        for line in lines.iter() {
            let len = line.len();
            if len > 0 {
                total_lines += 1;
                
                // Group line lengths by tens (80-89, 90-99, etc.)
                let group = (len / 10) * 10;
                *length_counts.entry(group).or_insert(0) += 1;
            }
        }
        
        if total_lines > 0 {
            // We don't need to find the most common line length group
            // as we're using the 95th percentile approach instead
            
            // Find 95th percentile line length
            let mut sorted_lengths: Vec<usize> = Vec::new();
            for line in lines.iter() {
                if !line.trim().is_empty() {
                    sorted_lengths.push(line.len());
                }
            }
            sorted_lengths.sort_unstable();
            
            let p95_index = (sorted_lengths.len() as f64 * 0.95) as usize;
            let p95_length = if p95_index < sorted_lengths.len() {
                sorted_lengths[p95_index]
            } else if !sorted_lengths.is_empty() {
                sorted_lengths[sorted_lengths.len() - 1]
            } else {
                0
            };
            
            let style_rule = StyleRule::MaxLineLength(p95_length);
            
            let mut patterns_lock = patterns.lock().unwrap();
            let mut pattern = StylePattern::new(style_rule, language);
            pattern.add_occurrence(None);
            
            // Add examples of longest lines
            let mut longest_lines: Vec<(usize, &str)> = lines.iter()
                .filter(|l| !l.trim().is_empty())
                .map(|&l| (l.len(), l))
                .collect();
                
            longest_lines.sort_by(|a, b| b.0.cmp(&a.0));
            
            for (len, line) in longest_lines.iter().take(3) {
                if *len > 50 {
                    pattern.examples.push(format!("Length {}: {}", len, if line.len() > 80 {
                        format!("{}...", &line[0..80])
                    } else {
                        line.to_string()
                    }));
                }
            }
            
            pattern.update_consistency(total_lines);
            patterns_lock.add_pattern(pattern);
        }
    }
}