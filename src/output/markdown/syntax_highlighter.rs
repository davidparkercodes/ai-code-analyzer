use std::collections::VecDeque;
use crate::output::style::{StyledText, Color, Style};

/// Helper function to format code blocks with syntax highlighting
pub fn format_code(code: &str, language: &str) -> String {
    match language {
        "rust" => highlight_rust(code),
        _ => StyledText::new(code)
            .foreground(Color::BrightWhite)
            .to_string()
    }
}

/// Struct to handle Rust syntax highlighting
struct RustHighlighter {
    line_parts: VecDeque<String>,
    current_part: String,
    in_string: bool,
    in_char: bool,
    in_comment: bool,
    keywords: Vec<&'static str>,
}

impl RustHighlighter {
    fn new() -> Self {
        RustHighlighter {
            line_parts: VecDeque::new(),
            current_part: String::new(),
            in_string: false,
            in_char: false,
            in_comment: false,
            keywords: vec![
                "as", "break", "const", "continue", "crate", "else", "enum", "extern",
                "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
                "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
                "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async",
                "await", "dyn"
            ],
        }
    }
    
    fn highlight_line(&mut self, line: &str) -> String {
        self.reset_state();
        
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let c = chars[i];
            
            if self.handle_string_literal(c, &chars, i) {
                // Character was handled by string literal logic
            } else if self.handle_char_literal(c) {
                // Character was handled by char literal logic
            } else if self.handle_comment(c, &chars, i) {
                i += 1; // Skip the second '/' in comment
            } else {
                self.handle_regular_character(c);
            }
            
            i += 1;
        }
        
        self.finalize_line_parts();
        self.join_line_parts()
    }
    
    fn reset_state(&mut self) {
        self.line_parts.clear();
        self.current_part.clear();
        self.in_string = false;
        self.in_char = false;
        self.in_comment = false;
    }
    
    fn handle_string_literal(&mut self, c: char, _chars: &[char], _i: usize) -> bool {
        if c == '"' && !self.in_char && !self.in_comment {
            if !self.in_string && !self.current_part.is_empty() {
                self.line_parts.push_back(self.current_part.clone());
                self.current_part.clear();
            }
            
            self.in_string = !self.in_string;
            self.current_part.push(c);
            
            if !self.in_string {
                self.line_parts.push_back(
                    StyledText::new(&self.current_part)
                        .foreground(Color::BrightGreen)
                        .to_string()
                );
                self.current_part.clear();
            }
            
            return true;
        }
        
        false
    }
    
    fn handle_char_literal(&mut self, c: char) -> bool {
        if c == '\'' && !self.in_string && !self.in_comment {
            if !self.in_char && !self.current_part.is_empty() {
                self.line_parts.push_back(self.current_part.clone());
                self.current_part.clear();
            }
            
            self.in_char = !self.in_char;
            self.current_part.push(c);
            
            if !self.in_char {
                self.line_parts.push_back(
                    StyledText::new(&self.current_part)
                        .foreground(Color::BrightGreen)
                        .to_string()
                );
                self.current_part.clear();
            }
            
            return true;
        }
        
        false
    }
    
    fn handle_comment(&mut self, c: char, chars: &[char], i: usize) -> bool {
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' && !self.in_string && !self.in_char && !self.in_comment {
            if !self.current_part.is_empty() {
                self.line_parts.push_back(self.current_part.clone());
                self.current_part.clear();
            }
            
            self.in_comment = true;
            self.current_part.push_str("//");
            
            return true;
        }
        
        false
    }
    
    fn handle_regular_character(&mut self, c: char) {
        self.current_part.push(c);
        
        // If we're in a string, char or comment, just continue collecting
        if self.in_string || self.in_char || self.in_comment {
            return;
        }
        
        // Check for keywords and tokens
        self.handle_token_boundary(c);
    }
    
    fn handle_token_boundary(&mut self, c: char) {
        let is_token_boundary = c.is_whitespace() || 
            c == '(' || c == ')' || c == '{' || c == '}' || 
            c == '[' || c == ']' || c == ';' || c == ',' || 
            c == '.' || c == ':';
            
        if is_token_boundary && self.current_part.len() > 1 {
            let word_part = &self.current_part[0..self.current_part.len() - 1];
            let delim_part = &self.current_part[self.current_part.len() - 1..];
            
            if self.keywords.contains(&word_part) {
                self.line_parts.push_back(
                    StyledText::new(word_part)
                        .foreground(Color::BrightMagenta)
                        .style(Style::Bold)
                        .to_string()
                );
                self.line_parts.push_back(delim_part.to_string());
            } else {
                self.line_parts.push_back(self.current_part.clone());
            }
            
            self.current_part.clear();
        }
    }
    
    fn finalize_line_parts(&mut self) {
        if self.current_part.is_empty() {
            return;
        }
        
        if self.in_string || self.in_char {
            self.line_parts.push_back(
                StyledText::new(&self.current_part)
                    .foreground(Color::BrightGreen)
                    .to_string()
            );
        } else if self.in_comment {
            self.line_parts.push_back(
                StyledText::new(&self.current_part)
                    .foreground(Color::BrightBlack)
                    .style(Style::Italic)
                    .to_string()
            );
        } else if self.keywords.contains(&&self.current_part[..]) {
            self.line_parts.push_back(
                StyledText::new(&self.current_part)
                    .foreground(Color::BrightMagenta)
                    .style(Style::Bold)
                    .to_string()
            );
        } else {
            self.line_parts.push_back(self.current_part.clone());
        }
    }
    
    fn join_line_parts(&self) -> String {
        let mut result = String::new();
        for part in &self.line_parts {
            result.push_str(part);
        }
        result
    }
}

/// Basic Rust syntax highlighting
pub fn highlight_rust(code: &str) -> String {
    let mut result = String::new();
    let mut highlighter = RustHighlighter::new();
    
    for line in code.lines() {
        let highlighted_line = highlighter.highlight_line(line);
        result.push_str(&highlighted_line);
        result.push('\n');
    }
    
    result
}