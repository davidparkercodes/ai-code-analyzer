use crate::style::models::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use regex::Regex;

const LINE_LENGTH_LIMIT: usize = 100;

pub struct StyleDetector {
    language_extensions: HashMap<String, String>,
    ignored_dirs: Vec<String>,
    ignored_files: Vec<String>,
}

impl Default for StyleDetector {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to count non-empty lines in a file
fn content_lines_count(file_path: &str) -> usize {
    if let Ok(content) = fs::read_to_string(file_path) {
        content.lines()
            .filter(|line| !line.trim().is_empty())
            .count()
    } else {
        0 // If can't read the file, assume it's a small file
    }
}

impl StyleDetector {
    pub fn new() -> Self {
        let mut detector = StyleDetector {
            language_extensions: HashMap::new(),
            ignored_dirs: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
            ],
            ignored_files: vec![
                ".DS_Store".to_string(), 
                "Cargo.lock".to_string(),
                "package-lock.json".to_string(),
            ],
        };
        
        // Initialize language extensions map with common extensions
        let common_extensions = [
            ("rs", "Rust"),
            ("js", "JavaScript"),
            ("jsx", "JavaScript"),
            ("ts", "TypeScript"),
            ("tsx", "TypeScript"),
            ("py", "Python"),
            ("java", "Java"),
            ("c", "C"),
            ("cpp", "C++"),
            ("h", "C"),
            ("hpp", "C++"),
            ("cs", "C#"),
            ("go", "Go"),
            ("rb", "Ruby"),
            ("php", "PHP"),
            ("html", "HTML"),
            ("css", "CSS"),
            ("scss", "SCSS"),
            ("md", "Markdown"),
            ("json", "JSON"),
            ("yaml", "YAML"),
            ("yml", "YAML"),
            ("toml", "TOML"),
        ];
        
        for (ext, lang) in common_extensions {
            detector.language_extensions.insert(ext.to_string(), lang.to_string());
        }
        
        detector
    }
    
    pub fn detect_styles<P: AsRef<Path>>(&self, path: P) -> Result<CodeStyleAnalysis, String> {
        let mut analysis = CodeStyleAnalysis::new();
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }
        
        if path.is_dir() {
            self.process_directory(path, &mut analysis)?;
        } else {
            self.process_file(path, &mut analysis)?;
        }
        
        self.compute_global_profile(&mut analysis);
        self.detect_inconsistencies(&mut analysis);
        
        Ok(analysis)
    }
    
    fn process_directory(&self, dir_path: &Path, analysis: &mut CodeStyleAnalysis) -> Result<(), String> {
        let entries = fs::read_dir(dir_path)
            .map_err(|e| format!("Failed to read directory {}: {}", dir_path.display(), e))?;
            
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
                
            // Skip ignored directories and files
            if (path.is_dir() && self.ignored_dirs.contains(&file_name.to_string())) ||
               (path.is_file() && self.ignored_files.contains(&file_name.to_string())) {
                continue;
            }
            
            if path.is_dir() {
                self.process_directory(&path, analysis)?;
            } else if path.is_file() {
                self.process_file(&path, analysis)?;
            }
        }
        
        Ok(())
    }
    
    fn process_file(&self, file_path: &Path, analysis: &mut CodeStyleAnalysis) -> Result<(), String> {
        // Get file extension
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
            
        // Skip if we don't recognize this file type
        if !self.language_extensions.contains_key(extension) {
            return Ok(());
        }
        
        let language = self.language_extensions.get(extension)
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());
            
        // Read file content
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
            
        // Create style profile for this file
        let style_profile = self.analyze_file_style(&content, &language);
        
        // Add profile to analysis
        if let Some(file_path_str) = file_path.to_str() {
            analysis.file_profiles.insert(file_path_str.to_string(), style_profile);
        }
        
        Ok(())
    }
    
    fn analyze_file_style(&self, content: &str, language: &str) -> StyleProfile {
        let mut profile = StyleProfile::new(language);
        
        // Detect indentation style
        let (indentation, indentation_violations) = self.detect_indentation(content);
        profile.indentation = indentation;
        profile.indentation_violation_lines = indentation_violations;
        
        // Detect brace style
        let (brace_style, brace_violations) = self.detect_brace_style(content, language);
        profile.brace_style = brace_style;
        profile.brace_style_violation_lines = brace_violations;
        
        // Analyze line metrics
        let (line_metrics, trailing_whitespace, whitespace_lines) = self.analyze_lines(content);
        profile.line_metrics = line_metrics;
        profile.trailing_whitespace_count = trailing_whitespace;
        profile.trailing_whitespace_lines = whitespace_lines;
        
        // Analyze function metrics
        profile.function_metrics = self.analyze_functions(content, language);
        
        // Analyze naming conventions
        profile.naming = self.analyze_naming_conventions(content, language);
        
        // Detect trailing semicolons (language specific)
        profile.has_trailing_semicolons = self.detect_trailing_semicolons(content, language);
        
        // Calculate comment ratio
        profile.comment_ratio = self.calculate_comment_ratio(content, language);
        
        profile
    }
    
    fn detect_indentation(&self, content: &str) -> (IndentationType, Vec<usize>) {
        // Track lines with inconsistent indentation
        let mut inconsistent_lines = Vec::new();
        
        // Count non-empty lines to see if we have enough data for indentation detection
        let non_empty_lines = content.lines()
            .filter(|line| !line.trim().is_empty())
            .count();
            
        // For very small files with just a few lines, indentation isn't meaningful
        if non_empty_lines < 5 {
            // For small files, detect basic indentation but don't report mixed/unknown
            // Look for consistent indentation in the few lines available
            let mut space_count = 0;
            let mut tab_count = 0;
            
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                
                if line.starts_with("\t") {
                    tab_count += 1;
                } else if line.starts_with(" ") {
                    space_count += 1;
                }
            }
            
            if tab_count > 0 && space_count == 0 {
                return (IndentationType::Tabs, inconsistent_lines);
            } else if space_count > 0 && tab_count == 0 {
                // Try to detect 2/4 spaces in small files
                for line in content.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    let leading_spaces = line.len() - line.trim_start().len();
                    if leading_spaces == 2 {
                        return (IndentationType::Spaces(2), inconsistent_lines);
                    } else if leading_spaces == 4 {
                        return (IndentationType::Spaces(4), inconsistent_lines);
                    } else if leading_spaces > 0 {
                        return (IndentationType::Spaces(leading_spaces), inconsistent_lines);
                    }
                }
            }
            
            // Default to the most common indentation for very small files
            return (IndentationType::Spaces(4), inconsistent_lines);
        }
        
        // Standard indentation detection for regular files
        let mut space_counts = HashMap::new();
        let mut has_tabs = false;
        let mut lines_with_tabs = Vec::new();
        let mut indentation_by_line = HashMap::new();
        
        // Get all lines first for easier context checking
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_number = i + 1;
            
            if line.trim().is_empty() {
                continue;
            }
            
            let leading_spaces = line.len() - line.trim_start().len();
            
            // Only consider lines that would reveal indentation - typically those with a non-trivial amount of code
            // and that start with whitespace
            let trimmed = line.trim();
            
            // Skip lines that are:
            // 1. Empty or have minimal content
            // 2. Part of a continued expression (ending with operators like ||, &&, +, etc.)
            // 3. Continuation of a chain/fluent API (.method().method())
            // 4. Just closing parentheses, braces, or brackets
            // Check if this line is part of a continuation or a method chain
            let is_continuation_line = i > 0 && {
                let prev_trimmed = lines[i-1].trim();
                prev_trimmed.ends_with('+') || 
                prev_trimmed.ends_with('-') || 
                prev_trimmed.ends_with('*') || 
                prev_trimmed.ends_with('/') || 
                prev_trimmed.ends_with('|') || 
                prev_trimmed.ends_with('&') || 
                prev_trimmed.ends_with(',') || 
                prev_trimmed.ends_with('.')
            };
            
            // Check if this line starts with a continuation character
            let starts_with_continuation = 
                trimmed.starts_with('.') || 
                trimmed.starts_with(')') || 
                trimmed.starts_with('}') || 
                trimmed.starts_with(']') || 
                trimmed.starts_with("||") || 
                trimmed.starts_with("&&");
            
            // Check if this line is a parameter in a method call or function call
            let is_parameter_line = 
                // Lines that are string literals, numbers, or identifiers often followed by commas
                (trimmed.starts_with('"') || 
                 trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) ||
                 trimmed.chars().next().is_some_and(|c| c.is_alphabetic())) &&
                (trimmed.ends_with(',') || trimmed.ends_with(')')) &&
                i > 0 && i < lines.len() - 1 &&
                // Previous line has a function call or method invocation pattern
                (lines[i-1].contains('(') || lines[i-1].trim().ends_with(','));
            let is_trivial = trimmed.len() <= 3 || trimmed.matches(|c| c != '}' && c != '{' && c != ')' && c != ']').count() <= 2;
            
            if !trimmed.is_empty() && 
               trimmed.len() < line.len() && 
               !is_trivial &&
               !is_continuation_line && 
               !starts_with_continuation && 
               !is_parameter_line &&
               !trimmed.ends_with("||") && 
               !trimmed.ends_with("&&") {
                
                if line.starts_with('\t') {
                    has_tabs = true;
                    lines_with_tabs.push(line_number);
                    indentation_by_line.insert(line_number, IndentationType::Tabs);
                } else if leading_spaces > 0 {
                    *space_counts.entry(leading_spaces).or_insert(0) += 1;
                    indentation_by_line.insert(line_number, IndentationType::Spaces(leading_spaces));
                }
            }
        }
        
        // Determine most common indentation
        if has_tabs {
            if !space_counts.is_empty() {
                // Track lines with inconsistent indentation in a mixed file
                for (line_number, indent_type) in &indentation_by_line {
                    match indent_type {
                        IndentationType::Tabs => {},  // Tabs are common in this file
                        IndentationType::Spaces(_) => inconsistent_lines.push(*line_number), // Spaces are inconsistent
                        _ => {}
                    }
                }
                return (IndentationType::Mixed, inconsistent_lines);
            } else {
                return (IndentationType::Tabs, inconsistent_lines);
            }
        }
        
        if space_counts.is_empty() {
            return (IndentationType::Unknown, inconsistent_lines);
        }
        
        // Find most common space count
        let (most_common, _) = space_counts.iter()
            .max_by_key(|&(_, count)| *count)
            .unwrap_or((&0, &0));
            
        // Check if it's a common indentation size (2, 4, 8)
        let common_spaces = *most_common;
        if common_spaces % 2 == 0 && common_spaces <= 8 {
            // Track lines with inconsistent indentation
            for (line_number, indent_type) in &indentation_by_line {
                match indent_type {
                    IndentationType::Spaces(n) if *n != common_spaces => {
                        inconsistent_lines.push(*line_number);
                    },
                    _ => {}
                }
            }
            (IndentationType::Spaces(common_spaces), inconsistent_lines)
        } else {
            // All lines with indentation are inconsistent in a Mixed file
            for line_number in indentation_by_line.keys() {
                inconsistent_lines.push(*line_number);
            }
            (IndentationType::Mixed, inconsistent_lines)
        }
    }
    
    fn detect_brace_style(&self, content: &str, language: &str) -> (BraceStyle, Vec<usize>) {
        // Track lines with inconsistent brace style
        let mut inconsistent_lines = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        // Count lines to see if we have enough data for brace style detection
        let line_count = lines.len();
        
        // Default to same line for very small files or non-code files
        if line_count < 5 {
            return (BraceStyle::SameLine, inconsistent_lines); // Default for Rust
        }
        
        // This is a simplified implementation that works for C-like languages
        match language {
            "Rust" | "JavaScript" | "TypeScript" | "Java" | "C" | "C++" | "C#" => {
                let mut same_line = 0;
                let mut next_line = 0;
                
                // Regex to detect patterns like ) { or "){" (same line)
                let same_line_pattern = Regex::new(r"\)\s*\{").unwrap();
                
                // Regex to detect patterns like ")\n{" (next line)
                let _next_line_pattern = Regex::new(r"\)\s*[\r\n]+\s*\{").unwrap();
                
                // Check each line for brace styles and track line numbers
                for (i, line) in lines.iter().enumerate() {
                    let line_number = i + 1;
                    
                    // Only consider actual function or method declarations for brace style
                    // This helps avoid false positives from things like if statements, loops, etc.
                    let trimmed = line.trim();
                    if (trimmed.starts_with("fn ") || 
                        trimmed.starts_with("pub fn ") || 
                        trimmed.contains(" fn ")) && 
                        trimmed.contains('(') {
                        
                        if same_line_pattern.is_match(line) {
                            same_line += 1;
                        }
                        
                        // For next line style, we need to check if this line has a closing parenthesis
                        // and the next line has an opening brace
                        if i < line_count - 1 && line.trim().ends_with(')') && lines[i+1].trim().starts_with('{') {
                            next_line += 1;
                            // Mark both this line and the next line
                            inconsistent_lines.push(line_number);
                            inconsistent_lines.push(line_number + 1);
                        }
                    }
                }
                
                // Clear inconsistent lines if the style is consistent
                if same_line > 0 && next_line > 0 {
                    if same_line > next_line * 2 {
                        // Same line is dominant, so mark only next line style as inconsistent
                        inconsistent_lines.clear();
                        for (i, line) in lines.iter().enumerate() {
                            let line_number = i + 1;
                            if i < line_count - 1 && line.trim().ends_with(')') && lines[i+1].trim().starts_with('{') {
                                inconsistent_lines.push(line_number);
                                inconsistent_lines.push(line_number + 1);
                            }
                        }
                        (BraceStyle::SameLine, inconsistent_lines)
                    } else if next_line > same_line * 2 {
                        // Next line is dominant, so mark only same line style as inconsistent
                        inconsistent_lines.clear();
                        for (i, line) in lines.iter().enumerate() {
                            let line_number = i + 1;
                            if same_line_pattern.is_match(line) {
                                inconsistent_lines.push(line_number);
                            }
                        }
                        (BraceStyle::NextLine, inconsistent_lines)
                    } else {
                        // Mixed style, keep all inconsistencies
                        (BraceStyle::Mixed, inconsistent_lines)
                    }
                } else if same_line > 0 {
                    // Clear inconsistencies since everything is consistent
                    (BraceStyle::SameLine, Vec::new())
                } else if next_line > 0 {
                    // Clear inconsistencies since everything is consistent
                    (BraceStyle::NextLine, Vec::new())
                } else {
                    // If we can't detect any braces, use the default for the language
                    match language {
                        "Rust" | "JavaScript" | "TypeScript" | "Java" => (BraceStyle::SameLine, Vec::new()),
                        "C" | "C++" | "C#" => (BraceStyle::NextLine, Vec::new()),
                        _ => (BraceStyle::Unknown, Vec::new()),
                    }
                }
            },
            // Set defaults for other languages
            "Python" | "Ruby" => (BraceStyle::SameLine, Vec::new()), // No braces, but use default
            _ => (BraceStyle::Unknown, Vec::new()),
        }
    }
    
    fn analyze_lines(&self, content: &str) -> (LineMetrics, usize, Vec<usize>) {
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return (
                LineMetrics {
                    avg_length: 0.0,
                    max_length: 0,
                    over_limit_count: 0,
                    long_line_numbers: Vec::new(),
                },
                0,
                Vec::new()
            );
        }
        
        let mut total_length = 0;
        let mut max_length = 0;
        let mut over_limit = 0;
        let mut trailing_whitespace = 0;
        let mut long_line_numbers = Vec::new();
        let mut whitespace_line_numbers = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            let line_number = i + 1; // Lines are 1-indexed
            let line_len = line.len();
            total_length += line_len;
            
            if line_len > max_length {
                max_length = line_len;
            }
            
            // Only report long lines that have meaningful content
            let trimmed = line.trim();
            if line_len > LINE_LENGTH_LIMIT && trimmed.len() > 10 {
                // Skip trivial lines that are just closing braces or similar
                let non_trivial_chars = trimmed.chars()
                    .filter(|&c| c != ' ' && c != '}' && c != ';' && c != ')' && c != '{')
                    .count();
                
                // Only report if there's meaningful content
                if non_trivial_chars > 3 {
                    over_limit += 1;
                    long_line_numbers.push(line_number);
                }
            }
            
            // Check trailing whitespace - but only mark non-empty and meaningful lines
            if !line.trim().is_empty() && line.trim().len() > 3 && (line.ends_with(' ') || line.ends_with('\t')) {
                trailing_whitespace += 1;
                whitespace_line_numbers.push(line_number);
            }
        }
        
        (
            LineMetrics {
                avg_length: total_length as f64 / lines.len() as f64,
                max_length,
                over_limit_count: over_limit,
                long_line_numbers,
            },
            trailing_whitespace,
            whitespace_line_numbers
        )
    }
    
    fn analyze_functions(&self, content: &str, language: &str) -> FunctionMetrics {
        match language {
            "Rust" => self.analyze_rust_functions(content),
            "JavaScript" | "TypeScript" => self.analyze_js_functions(content),
            "Python" => self.analyze_python_functions(content),
            _ => FunctionMetrics {
                avg_length: 0.0,
                max_length: 0,
                avg_params: 0.0,
                max_params: 0,
            },
        }
    }
    
    fn analyze_rust_functions(&self, content: &str) -> FunctionMetrics {
        // Very simplified Rust function detection
        let fn_pattern = Regex::new(r"fn\s+(\w+)\s*\(([^)]*)\)").unwrap();
        let mut functions = Vec::new();
        
        for cap in fn_pattern.captures_iter(content) {
            let params = cap.get(2).map_or("", |m| m.as_str());
            let param_count = if params.trim().is_empty() {
                0
            } else {
                params.split(',').count()
            };
            
            functions.push((0, param_count)); // Placeholder for function length
        }
        
        // Very crude approximation since proper analysis requires full parsing
        if functions.is_empty() {
            return FunctionMetrics {
                avg_length: 0.0,
                max_length: 0,
                avg_params: 0.0,
                max_params: 0,
            };
        }
        
        let avg_params = functions.iter().map(|(_, p)| *p).sum::<usize>() as f64 / functions.len() as f64;
        let max_params = functions.iter().map(|(_, p)| *p).max().unwrap_or(0);
        
        FunctionMetrics {
            avg_length: 20.0, // Placeholder - proper analysis requires AST parsing
            max_length: 100,  // Placeholder
            avg_params,
            max_params,
        }
    }
    
    fn analyze_js_functions(&self, content: &str) -> FunctionMetrics {
        // Very simplified JS/TS function detection
        let fn_patterns = [
            Regex::new(r"function\s+(\w+)\s*\(([^)]*)\)").unwrap(),
            Regex::new(r"(\w+)\s*=\s*function\s*\(([^)]*)\)").unwrap(),
            Regex::new(r"(\w+)\s*:\s*function\s*\(([^)]*)\)").unwrap(),
            Regex::new(r"(\w+)\s*=\s*\(([^)]*)\)\s*=>").unwrap(),
            Regex::new(r"const\s+(\w+)\s*=\s*\(([^)]*)\)\s*=>").unwrap(),
        ];
        
        let mut functions = Vec::new();
        
        for pattern in &fn_patterns {
            for cap in pattern.captures_iter(content) {
                let params = cap.get(2).map_or("", |m| m.as_str());
                let param_count = if params.trim().is_empty() {
                    0
                } else {
                    params.split(',').count()
                };
                
                functions.push((0, param_count)); // Placeholder for function length
            }
        }
        
        if functions.is_empty() {
            return FunctionMetrics {
                avg_length: 0.0,
                max_length: 0,
                avg_params: 0.0,
                max_params: 0,
            };
        }
        
        let avg_params = functions.iter().map(|(_, p)| *p).sum::<usize>() as f64 / functions.len() as f64;
        let max_params = functions.iter().map(|(_, p)| *p).max().unwrap_or(0);
        
        FunctionMetrics {
            avg_length: 15.0, // Placeholder
            max_length: 80,   // Placeholder
            avg_params,
            max_params,
        }
    }
    
    fn analyze_python_functions(&self, content: &str) -> FunctionMetrics {
        // Simple Python function detection
        let fn_pattern = Regex::new(r"def\s+(\w+)\s*\(([^)]*)\)").unwrap();
        let mut functions = Vec::new();
        
        for cap in fn_pattern.captures_iter(content) {
            let params = cap.get(2).map_or("", |m| m.as_str());
            let param_count = if params.trim().is_empty() {
                0
            } else {
                params.split(',').count()
            };
            
            functions.push((0, param_count));
        }
        
        if functions.is_empty() {
            return FunctionMetrics {
                avg_length: 0.0,
                max_length: 0,
                avg_params: 0.0,
                max_params: 0,
            };
        }
        
        let avg_params = functions.iter().map(|(_, p)| *p).sum::<usize>() as f64 / functions.len() as f64;
        let max_params = functions.iter().map(|(_, p)| *p).max().unwrap_or(0);
        
        FunctionMetrics {
            avg_length: 15.0, // Placeholder
            max_length: 80,   // Placeholder
            avg_params,
            max_params,
        }
    }
    
    fn analyze_naming_conventions(&self, content: &str, language: &str) -> HashMap<String, NamingConvention> {
        let mut result = HashMap::new();
        
        match language {
            "Rust" => {
                // Variables
                result.insert("variables".to_string(), self.detect_variable_convention(content, language));
                
                // Functions
                let fn_pattern = Regex::new(r"fn\s+(\w+)").unwrap();
                let function_names: Vec<String> = fn_pattern.captures_iter(content)
                    .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
                    .collect();
                    
                if !function_names.is_empty() {
                    result.insert("functions".to_string(), self.determine_naming_convention(&function_names));
                }
                
                // Structs/Enums
                let struct_pattern = Regex::new(r"struct\s+(\w+)").unwrap();
                let enum_pattern = Regex::new(r"enum\s+(\w+)").unwrap();
                
                let mut type_names = Vec::new();
                
                for cap in struct_pattern.captures_iter(content) {
                    if let Some(m) = cap.get(1) {
                        type_names.push(m.as_str().to_string());
                    }
                }
                
                for cap in enum_pattern.captures_iter(content) {
                    if let Some(m) = cap.get(1) {
                        type_names.push(m.as_str().to_string());
                    }
                }
                
                if !type_names.is_empty() {
                    result.insert("types".to_string(), self.determine_naming_convention(&type_names));
                }
            },
            "JavaScript" | "TypeScript" => {
                // Variables
                result.insert("variables".to_string(), self.detect_variable_convention(content, language));
                
                // Functions
                let fn_pattern = Regex::new(r"function\s+(\w+)").unwrap();
                let function_names: Vec<String> = fn_pattern.captures_iter(content)
                    .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
                    .collect();
                    
                if !function_names.is_empty() {
                    result.insert("functions".to_string(), self.determine_naming_convention(&function_names));
                }
                
                // Classes
                if language == "TypeScript" {
                    let class_pattern = Regex::new(r"class\s+(\w+)").unwrap();
                    let class_names: Vec<String> = class_pattern.captures_iter(content)
                        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
                        .collect();
                        
                    if !class_names.is_empty() {
                        result.insert("classes".to_string(), self.determine_naming_convention(&class_names));
                    }
                    
                    // Interfaces
                    let interface_pattern = Regex::new(r"interface\s+(\w+)").unwrap();
                    let interface_names: Vec<String> = interface_pattern.captures_iter(content)
                        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
                        .collect();
                        
                    if !interface_names.is_empty() {
                        result.insert("interfaces".to_string(), self.determine_naming_convention(&interface_names));
                    }
                }
            },
            _ => {
                // Basic variable convention for other languages
                result.insert("variables".to_string(), self.detect_variable_convention(content, language));
            }
        }
        
        result
    }
    
    fn detect_variable_convention(&self, content: &str, language: &str) -> NamingConvention {
        let variable_names = match language {
            "Rust" => {
                let let_pattern = Regex::new(r"let\s+(\w+)").unwrap();
                let mut names = Vec::new();
                
                for cap in let_pattern.captures_iter(content) {
                    if let Some(m) = cap.get(1) {
                        names.push(m.as_str().to_string());
                    }
                }
                
                names
            },
            "JavaScript" | "TypeScript" => {
                let var_patterns = [
                    Regex::new(r"var\s+(\w+)").unwrap(),
                    Regex::new(r"let\s+(\w+)").unwrap(),
                    Regex::new(r"const\s+(\w+)").unwrap(),
                ];
                
                let mut names = Vec::new();
                
                for pattern in &var_patterns {
                    for cap in pattern.captures_iter(content) {
                        if let Some(m) = cap.get(1) {
                            names.push(m.as_str().to_string());
                        }
                    }
                }
                
                names
            },
            "Python" => {
                // Very simple var detection, not perfect
                let var_pattern = Regex::new(r"(\w+)\s*=").unwrap();
                let mut names = Vec::new();
                
                for cap in var_pattern.captures_iter(content) {
                    if let Some(m) = cap.get(1) {
                        let name = m.as_str();
                        // Filter out some common non-variable patterns
                        if !["if", "while", "for", "def", "class"].contains(&name) {
                            names.push(name.to_string());
                        }
                    }
                }
                
                names
            },
            _ => Vec::new(),
        };
        
        if variable_names.is_empty() {
            return NamingConvention::Unknown;
        }
        
        self.determine_naming_convention(&variable_names)
    }
    
    fn determine_naming_convention(&self, names: &[String]) -> NamingConvention {
        let mut camel_case = 0;
        let mut snake_case = 0;
        let mut pascal_case = 0;
        let mut kebab_case = 0;
        
        for name in names {
            if name.contains('_') {
                snake_case += 1;
            } else if name.contains('-') {
                kebab_case += 1;
            } else if let Some(first_char) = name.chars().next() {
                if first_char.is_uppercase() {
                    pascal_case += 1;
                } else if name.chars().any(|c| c.is_uppercase()) {
                    camel_case += 1;
                } else {
                    // Lower case, no separators - could be any style with single word
                    // We'll increment snake_case as it's most common for simple vars
                    snake_case += 1;
                }
            }
        }
        
        // Determine which convention is most common
        let total = names.len();
        let threshold = (total as f64 * 0.75) as usize; // 75% threshold for consistency
        
        if snake_case > threshold {
            NamingConvention::SnakeCase
        } else if camel_case > threshold {
            NamingConvention::CamelCase
        } else if pascal_case > threshold {
            NamingConvention::PascalCase
        } else if kebab_case > threshold {
            NamingConvention::KebabCase
        } else {
            NamingConvention::Mixed
        }
    }
    
    fn detect_trailing_semicolons(&self, content: &str, language: &str) -> Option<bool> {
        match language {
            "JavaScript" | "TypeScript" => {
                let mut with_semi = 0;
                let mut without_semi = 0;
                
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.ends_with('{') || trimmed.ends_with('}') {
                        continue;
                    }
                    
                    if trimmed.ends_with(';') {
                        with_semi += 1;
                    } else {
                        without_semi += 1;
                    }
                }
                
                if with_semi + without_semi > 0 {
                    Some(with_semi > without_semi)
                } else {
                    None
                }
            },
            _ => None, // Other languages like Rust have mandatory semicolons
        }
    }
    
    fn calculate_comment_ratio(&self, content: &str, language: &str) -> f64 {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return 0.0;
        }
        
        let mut comment_lines = 0;
        
        match language {
            "Rust" | "C" | "C++" | "JavaScript" | "TypeScript" | "Java" | "C#" => {
                let mut in_block_comment = false;
                
                for line in &lines {
                    let trimmed = line.trim();
                    
                    if in_block_comment {
                        comment_lines += 1;
                        if trimmed.contains("*/") {
                            in_block_comment = false;
                        }
                    } else if trimmed.starts_with("//") {
                        comment_lines += 1;
                    } else if trimmed.contains("/*") {
                        comment_lines += 1;
                        in_block_comment = true;
                        
                        if trimmed.contains("*/") {
                            in_block_comment = false;
                        }
                    }
                }
            },
            "Python" => {
                let mut in_block_comment = false;
                
                for line in &lines {
                    let trimmed = line.trim();
                    
                    if trimmed.starts_with("#") {
                        comment_lines += 1;
                    } else if trimmed.starts_with("'''") || trimmed.starts_with("\"\"\"") {
                        comment_lines += 1;
                        in_block_comment = !in_block_comment;
                    } else if in_block_comment {
                        comment_lines += 1;
                        if trimmed.ends_with("'''") || trimmed.ends_with("\"\"\"") {
                            in_block_comment = false;
                        }
                    }
                }
            },
            _ => {},
        }
        
        comment_lines as f64 / lines.len() as f64
    }
    
    fn compute_global_profile(&self, analysis: &mut CodeStyleAnalysis) {
        if analysis.file_profiles.is_empty() {
            return;
        }
        
        // Count style frequencies
        let mut indentation_counts: HashMap<String, usize> = HashMap::new();
        let mut brace_style_counts: HashMap<String, usize> = HashMap::new();
        let mut naming_conventions: HashMap<String, HashMap<String, usize>> = HashMap::new();
        let mut semicolon_counts = HashMap::new();
        let mut language_counts: HashMap<String, usize> = HashMap::new();
        
        let mut total_line_length = 0.0;
        let mut max_line_length = 0;
        let mut over_limit_lines = 0;
        
        let mut total_func_length = 0.0;
        let mut max_func_length = 0;
        let mut total_params = 0.0;
        let mut max_params = 0;
        
        let mut total_comment_ratio = 0.0;
        let mut total_trailing_whitespace = 0;
        
        let profile_count = analysis.file_profiles.len();
        
        for profile in analysis.file_profiles.values() {
            // Count file types
            if !profile.file_type.is_empty() {
                *language_counts.entry(profile.file_type.clone()).or_insert(0) += 1;
            }
            
            // Count indentation types
            let indent_key = match &profile.indentation {
                IndentationType::Spaces(n) => format!("spaces-{}", n),
                IndentationType::Tabs => "tabs".to_string(),
                IndentationType::Mixed => "mixed".to_string(),
                IndentationType::Unknown => "unknown".to_string(),
            };
            *indentation_counts.entry(indent_key).or_insert(0) += 1;
            
            // Count brace styles
            let brace_key = match &profile.brace_style {
                BraceStyle::SameLine => "same-line".to_string(),
                BraceStyle::NextLine => "next-line".to_string(),
                BraceStyle::Mixed => "mixed".to_string(),
                BraceStyle::Unknown => "unknown".to_string(),
            };
            *brace_style_counts.entry(brace_key).or_insert(0) += 1;
            
            // Accumulate line metrics
            total_line_length += profile.line_metrics.avg_length;
            max_line_length = max_line_length.max(profile.line_metrics.max_length);
            over_limit_lines += profile.line_metrics.over_limit_count;
            
            // Accumulate function metrics
            total_func_length += profile.function_metrics.avg_length;
            max_func_length = max_func_length.max(profile.function_metrics.max_length);
            total_params += profile.function_metrics.avg_params;
            max_params = max_params.max(profile.function_metrics.max_params);
            
            // Count naming conventions
            for (category, convention) in &profile.naming {
                let convention_map = naming_conventions.entry(category.clone()).or_insert_with(HashMap::new);
                
                let convention_key = match convention {
                    NamingConvention::CamelCase => "camelCase".to_string(),
                    NamingConvention::SnakeCase => "snake_case".to_string(),
                    NamingConvention::PascalCase => "PascalCase".to_string(),
                    NamingConvention::KebabCase => "kebab-case".to_string(),
                    NamingConvention::Mixed => "mixed".to_string(),
                    NamingConvention::Unknown => "unknown".to_string(),
                };
                
                *convention_map.entry(convention_key).or_insert(0) += 1;
            }
            
            // Count semicolon usage
            if let Some(has_semicolons) = profile.has_trailing_semicolons {
                let semi_key = if has_semicolons { "yes" } else { "no" };
                *semicolon_counts.entry(semi_key.to_string()).or_insert(0) += 1;
            }
            
            // Accumulate comment ratio
            total_comment_ratio += profile.comment_ratio;
            
            // Accumulate trailing whitespace
            total_trailing_whitespace += profile.trailing_whitespace_count;
        }
        
        // Create the global profile
        let mut global = StyleProfile::default();
        
        // Determine most common language
        if let Some((language, _)) = language_counts.iter().max_by_key(|&(_, count)| *count) {
            global.file_type = language.clone();
        }
        
        // Determine most common indentation
        if let Some((indent_key, _)) = indentation_counts.iter().max_by_key(|&(_, count)| *count) {
            if indent_key.starts_with("spaces-") {
                if let Ok(spaces) = indent_key[7..].parse::<usize>() {
                    global.indentation = IndentationType::Spaces(spaces);
                }
            } else if indent_key == "tabs" {
                global.indentation = IndentationType::Tabs;
            } else if indent_key == "mixed" {
                global.indentation = IndentationType::Mixed;
            }
        }
        
        // Determine most common brace style
        if let Some((brace_key, _)) = brace_style_counts.iter().max_by_key(|&(_, count)| *count) {
            if brace_key == "same-line" {
                global.brace_style = BraceStyle::SameLine;
            } else if brace_key == "next-line" {
                global.brace_style = BraceStyle::NextLine;
            } else if brace_key == "mixed" {
                global.brace_style = BraceStyle::Mixed;
            }
        }
        
        // Calculate global line metrics
        global.line_metrics = LineMetrics {
            avg_length: total_line_length / profile_count as f64,
            max_length: max_line_length,
            over_limit_count: over_limit_lines,
            long_line_numbers: Vec::new(), // Global profile doesn't track specific lines
        };
        
        // Calculate global function metrics
        global.function_metrics = FunctionMetrics {
            avg_length: total_func_length / profile_count as f64,
            max_length: max_func_length,
            avg_params: total_params / profile_count as f64,
            max_params,
        };
        
        // Determine naming conventions for global profile
        for (category, conventions) in naming_conventions {
            if let Some((convention, _)) = conventions.iter().max_by_key(|&(_, count)| *count) {
                let naming_style = match convention.as_str() {
                    "camelCase" => NamingConvention::CamelCase,
                    "snake_case" => NamingConvention::SnakeCase,
                    "PascalCase" => NamingConvention::PascalCase,
                    "kebab-case" => NamingConvention::KebabCase,
                    "mixed" => NamingConvention::Mixed,
                    _ => NamingConvention::Unknown,
                };
                
                global.naming.insert(category, naming_style);
            }
        }
        
        // Determine semicolon usage
        if let Some((semi_key, _)) = semicolon_counts.iter().max_by_key(|&(_, count)| *count) {
            global.has_trailing_semicolons = Some(semi_key == "yes");
        }
        
        // Calculate comment ratio
        global.comment_ratio = total_comment_ratio / profile_count as f64;
        
        // Set trailing whitespace
        global.trailing_whitespace_count = total_trailing_whitespace;
        
        // Set the global profile in the analysis
        analysis.global_profile = global;
    }
    
    fn detect_inconsistencies(&self, analysis: &mut CodeStyleAnalysis) -> f64 {
        if analysis.file_profiles.is_empty() {
            return 1.0; // Perfect score if no files
        }
        
        // Clone the global profile to avoid borrowing issues
        let global = analysis.global_profile.clone();
        let mut inconsistencies = Vec::new();
        let mut inconsistency_count = 0;
        let mut total_checks = 0;
        
        for (file_path, profile) in &analysis.file_profiles {
            // Calculate the number of non-empty lines in the file
            let line_count = content_lines_count(file_path);
            let is_small_file = line_count < 5;
            
            // Check indentation - skip for very small files (like mod.rs)
            if !is_small_file && 
               profile.indentation != global.indentation && 
               global.indentation != IndentationType::Unknown {
                total_checks += 1;
                
                let expected = match &global.indentation {
                    IndentationType::Spaces(n) => format!("{} spaces", n),
                    IndentationType::Tabs => "tabs".to_string(),
                    _ => "consistent indentation".to_string(),
                };
                
                let actual = match &profile.indentation {
                    IndentationType::Spaces(n) => format!("{} spaces", n),
                    IndentationType::Tabs => "tabs".to_string(),
                    IndentationType::Mixed => "mixed indentation".to_string(),
                    IndentationType::Unknown => "unknown indentation".to_string(),
                };
                
                // If we have specific line numbers for indentation violations, create an inconsistency for each
                if !profile.indentation_violation_lines.is_empty() {
                    // Get first 5 violation lines (to avoid too many repeated items)
                    let first_violations = profile.indentation_violation_lines.iter()
                        .take(5)
                        .copied()
                        .collect::<Vec<_>>();
                
                    for &line_number in &first_violations {
                        inconsistencies.push(StyleInconsistency {
                            file_path: file_path.clone(),
                            line_number: Some(line_number),
                            description: format!("Inconsistent indentation: expected {}, found {}", expected, actual),
                            severity: InconsistencySeverity::Medium,
                        });
                    }
                    
                    // If there are more violations than we're showing
                    if profile.indentation_violation_lines.len() > first_violations.len() {
                        inconsistencies.push(StyleInconsistency {
                            file_path: file_path.clone(),
                            line_number: None,
                            description: format!(
                                "And {} more lines with inconsistent indentation", 
                                profile.indentation_violation_lines.len() - first_violations.len()
                            ),
                            severity: InconsistencySeverity::Medium,
                        });
                    }
                } else {
                    // No specific lines identified, add a general inconsistency
                    inconsistencies.push(StyleInconsistency {
                        file_path: file_path.clone(),
                        line_number: None,
                        description: format!("Inconsistent indentation: expected {}, found {}", expected, actual),
                        severity: InconsistencySeverity::Medium,
                    });
                }
                
                inconsistency_count += 1;
            }
            
            // Check brace style - skip for very small files
            if !is_small_file && 
               profile.brace_style != global.brace_style && 
               global.brace_style != BraceStyle::Unknown && 
               profile.brace_style != BraceStyle::Unknown {
                total_checks += 1;
                
                let expected = match &global.brace_style {
                    BraceStyle::SameLine => "same line",
                    BraceStyle::NextLine => "next line",
                    _ => "consistent style",
                };
                
                let actual = match &profile.brace_style {
                    BraceStyle::SameLine => "same line",
                    BraceStyle::NextLine => "next line",
                    BraceStyle::Mixed => "mixed style",
                    BraceStyle::Unknown => "unknown style",
                };
                
                // If we have specific line numbers for brace style violations, create an inconsistency for each
                if !profile.brace_style_violation_lines.is_empty() {
                    // Get first 5 violation lines (to avoid too many repeated items)
                    let first_violations = profile.brace_style_violation_lines.iter()
                        .take(5)
                        .copied()
                        .collect::<Vec<_>>();
                
                    for &line_number in &first_violations {
                        inconsistencies.push(StyleInconsistency {
                            file_path: file_path.clone(),
                            line_number: Some(line_number),
                            description: format!("Inconsistent brace style: expected {}, found {}", expected, actual),
                            severity: InconsistencySeverity::Low,
                        });
                    }
                    
                    // If there are more violations than we're showing
                    if profile.brace_style_violation_lines.len() > first_violations.len() {
                        inconsistencies.push(StyleInconsistency {
                            file_path: file_path.clone(),
                            line_number: None,
                            description: format!(
                                "And {} more lines with inconsistent brace style", 
                                profile.brace_style_violation_lines.len() - first_violations.len()
                            ),
                            severity: InconsistencySeverity::Low,
                        });
                    }
                } else {
                    // No specific lines identified, add a general inconsistency
                    inconsistencies.push(StyleInconsistency {
                        file_path: file_path.clone(),
                        line_number: None,
                        description: format!("Inconsistent brace style: expected {}, found {}", expected, actual),
                        severity: InconsistencySeverity::Low,
                    });
                }
                
                inconsistency_count += 1;
            }
            
            // Check line length - small files may still have long lines
            if profile.line_metrics.over_limit_count > 0 {
                total_checks += 1;
                
                // If we have specific line numbers for long lines, create an inconsistency for each
                if !profile.line_metrics.long_line_numbers.is_empty() {
                    // Get first 5 long lines (to avoid too many repeated items)
                    let first_long_lines = profile.line_metrics.long_line_numbers.iter()
                        .take(5)
                        .copied()
                        .collect::<Vec<_>>();
                
                    for &line_number in &first_long_lines {
                        inconsistencies.push(StyleInconsistency {
                            file_path: file_path.clone(),
                            line_number: Some(line_number),
                            description: format!(
                                "Line exceeds recommended length limit of {} characters", 
                                LINE_LENGTH_LIMIT
                            ),
                            severity: InconsistencySeverity::Low,
                        });
                    }
                    
                    // If there are more long lines than we're showing
                    if profile.line_metrics.long_line_numbers.len() > first_long_lines.len() {
                        inconsistencies.push(StyleInconsistency {
                            file_path: file_path.clone(),
                            line_number: None,
                            description: format!(
                                "And {} more lines exceed the length limit", 
                                profile.line_metrics.long_line_numbers.len() - first_long_lines.len()
                            ),
                            severity: InconsistencySeverity::Low,
                        });
                    }
                } else {
                    // No specific lines identified, add a general inconsistency
                    inconsistencies.push(StyleInconsistency {
                        file_path: file_path.clone(),
                        line_number: None,
                        description: format!(
                            "File contains {} lines over the recommended length limit ({})", 
                            profile.line_metrics.over_limit_count,
                            LINE_LENGTH_LIMIT
                        ),
                        severity: InconsistencySeverity::Low,
                    });
                }
                
                inconsistency_count += 1;
            }
            
            // Check naming conventions - only check for larger files
            if !is_small_file {
                for (category, convention) in &global.naming {
                    if let Some(file_convention) = profile.naming.get(category) {
                        if file_convention != convention && 
                           *file_convention != NamingConvention::Unknown && 
                           *convention != NamingConvention::Unknown {
                            total_checks += 1;
                            
                            let expected = match convention {
                                NamingConvention::CamelCase => "camelCase",
                                NamingConvention::SnakeCase => "snake_case",
                                NamingConvention::PascalCase => "PascalCase",
                                NamingConvention::KebabCase => "kebab-case",
                                _ => "consistent naming",
                            };
                            
                            let actual = match file_convention {
                                NamingConvention::CamelCase => "camelCase",
                                NamingConvention::SnakeCase => "snake_case",
                                NamingConvention::PascalCase => "PascalCase",
                                NamingConvention::KebabCase => "kebab-case",
                                NamingConvention::Mixed => "mixed conventions",
                                NamingConvention::Unknown => "unknown convention",
                            };
                            
                            inconsistencies.push(StyleInconsistency {
                                file_path: file_path.clone(),
                                line_number: None,
                                description: format!(
                                    "Inconsistent naming for {}: expected {}, found {}", 
                                    category, expected, actual
                                ),
                                severity: InconsistencySeverity::Medium,
                            });
                            
                            inconsistency_count += 1;
                        }
                    }
                }
            }
            
            // Check semicolon usage (only for larger files)
            if !is_small_file {
                if let (Some(global_semi), Some(file_semi)) = (global.has_trailing_semicolons, profile.has_trailing_semicolons) {
                    if global_semi != file_semi {
                        total_checks += 1;
                        
                        inconsistencies.push(StyleInconsistency {
                            file_path: file_path.clone(),
                            line_number: None,
                            description: format!(
                                "Inconsistent semicolon usage: expected {}, found {}", 
                                if global_semi { "semicolons" } else { "no semicolons" },
                                if file_semi { "semicolons" } else { "no semicolons" }
                            ),
                            severity: InconsistencySeverity::Low,
                        });
                        
                        inconsistency_count += 1;
                    }
                }
            }
            
            // Check trailing whitespace - this can apply to files of any size
            if profile.trailing_whitespace_count > 0 {
                total_checks += 1;
                
                // If we have specific line numbers for trailing whitespace, create an inconsistency for each
                if !profile.trailing_whitespace_lines.is_empty() {
                    // Get first 5 trailing whitespace lines (to avoid too many repeated items)
                    let first_whitespace_lines = profile.trailing_whitespace_lines.iter()
                        .take(5)
                        .copied()
                        .collect::<Vec<_>>();
                
                    for &line_number in &first_whitespace_lines {
                        inconsistencies.push(StyleInconsistency {
                            file_path: file_path.clone(),
                            line_number: Some(line_number),
                            description: "Line contains trailing whitespace".to_string(),
                            severity: InconsistencySeverity::Low,
                        });
                    }
                    
                    // If there are more whitespace lines than we're showing
                    if profile.trailing_whitespace_lines.len() > first_whitespace_lines.len() {
                        inconsistencies.push(StyleInconsistency {
                            file_path: file_path.clone(),
                            line_number: None,
                            description: format!(
                                "And {} more lines with trailing whitespace", 
                                profile.trailing_whitespace_lines.len() - first_whitespace_lines.len()
                            ),
                            severity: InconsistencySeverity::Low,
                        });
                    }
                } else {
                    // No specific lines identified, add a general inconsistency
                    inconsistencies.push(StyleInconsistency {
                        file_path: file_path.clone(),
                        line_number: None,
                        description: format!("File contains {} lines with trailing whitespace", profile.trailing_whitespace_count),
                        severity: InconsistencySeverity::Low,
                    });
                }
                
                inconsistency_count += 1;
            }
            
            // We could add more checks here for other style aspects
        }
        
        // Calculate consistency score (0-1)
        let consistency_score = if total_checks == 0 {
            1.0
        } else {
            1.0 - (inconsistency_count as f64 / total_checks as f64)
        };
        
        // Update the analysis with inconsistencies and score
        analysis.inconsistencies = inconsistencies;
        analysis.consistency_score = consistency_score;
        
        consistency_score
    }
}