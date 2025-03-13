use std::collections::VecDeque;
use crate::output::style::{StyledText, Color, Style};

/// Helper function to format code blocks with syntax highlighting
pub fn format_code(code: &str, language: &str) -> String {
    // Basic code formatting - we could add more complex syntax highlighting in the future
    match language {
        "rust" => highlight_rust(code),
        _ => StyledText::new(code)
            .foreground(Color::BrightWhite)
            .to_string()
    }
}

/// Basic Rust syntax highlighting
pub fn highlight_rust(code: &str) -> String {
    let mut result = String::new();
    
    // Define rust keywords
    let keywords = vec![
        "as", "break", "const", "continue", "crate", "else", "enum", "extern",
        "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
        "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
        "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async",
        "await", "dyn"
    ];
    
    // Split by lines to maintain formatting
    for line in code.lines() {
        let mut line_parts = VecDeque::new();
        let mut current_part = String::new();
        let mut in_string = false;
        let mut in_char = false;
        let mut in_comment = false;
        
        // Process each character
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let c = chars[i];
            
            // Handle string literals
            if c == '"' && !in_char && !in_comment {
                if !in_string && !current_part.is_empty() {
                    line_parts.push_back(current_part);
                    current_part = String::new();
                }
                
                in_string = !in_string;
                current_part.push(c);
                
                if !in_string {
                    line_parts.push_back(StyledText::new(&current_part)
                        .foreground(Color::BrightGreen)
                        .to_string());
                    current_part = String::new();
                }
            }
            // Handle char literals
            else if c == '\'' && !in_string && !in_comment {
                if !in_char && !current_part.is_empty() {
                    line_parts.push_back(current_part);
                    current_part = String::new();
                }
                
                in_char = !in_char;
                current_part.push(c);
                
                if !in_char {
                    line_parts.push_back(StyledText::new(&current_part)
                        .foreground(Color::BrightGreen)
                        .to_string());
                    current_part = String::new();
                }
            }
            // Handle comments
            else if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' && !in_string && !in_char && !in_comment {
                if !current_part.is_empty() {
                    line_parts.push_back(current_part);
                    current_part = String::new();
                }
                
                in_comment = true;
                current_part.push_str("//");
                i += 1; // Skip the second '/'
            }
            else {
                current_part.push(c);
                
                // If we're in a string, char or comment, just continue collecting
                if in_string || in_char || in_comment {
                    // Do nothing special
                }
                // Otherwise, check for keywords and tokens
                else if c.is_whitespace() || c == '(' || c == ')' || c == '{' || c == '}' || 
                        c == '[' || c == ']' || c == ';' || c == ',' || c == '.' || c == ':' {
                    
                    if current_part.len() > 1 { // More than just the delimiter
                        let word_part = &current_part[0..current_part.len() - 1];
                        let delim_part = &current_part[current_part.len() - 1..];
                        
                        if keywords.contains(&word_part) {
                            line_parts.push_back(StyledText::new(word_part)
                                .foreground(Color::BrightMagenta)
                                .style(Style::Bold)
                                .to_string());
                            line_parts.push_back(delim_part.to_string());
                        } else {
                            line_parts.push_back(current_part.clone());
                        }
                        
                        current_part = String::new();
                    }
                }
            }
            
            i += 1;
        }
        
        // Handle any remaining content
        if !current_part.is_empty() {
            if in_string || in_char {
                line_parts.push_back(StyledText::new(&current_part)
                    .foreground(Color::BrightGreen)
                    .to_string());
            } else if in_comment {
                line_parts.push_back(StyledText::new(&current_part)
                    .foreground(Color::BrightBlack)
                    .style(Style::Italic)
                    .to_string());
            } else if keywords.contains(&&current_part[..]) {
                line_parts.push_back(StyledText::new(&current_part)
                    .foreground(Color::BrightMagenta)
                    .style(Style::Bold)
                    .to_string());
            } else {
                line_parts.push_back(current_part);
            }
        }
        
        // Join the line parts and add to result
        for part in line_parts {
            result.push_str(&part);
        }
        result.push('\n');
    }
    
    result
}