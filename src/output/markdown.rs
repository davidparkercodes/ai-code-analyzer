use pulldown_cmark::{Event, Parser, Tag};
use terminal_size::{Width, terminal_size};
use std::collections::VecDeque;

use crate::output::style::{StyledText, Color, Style, ThemeColors};

/// Format markdown text for terminal display
pub fn format_markdown(markdown_text: &str) -> String {
    let parser = Parser::new(markdown_text);
    let mut output = String::new();
    let mut format_stack = Vec::new();
    let mut list_stack = Vec::new();
    let mut code_block = false;
    let mut code_language = String::new();
    let terminal_width = get_terminal_width();
    let mut pending_newlines = 0;

    for event in parser {
        match event {
            Event::Start(tag) => {
                flush_newlines(&mut output, &mut pending_newlines);
                match tag {
                    Tag::Heading(level, _, _) => {
                        let level_num = level as usize;
                        if !output.is_empty() {
                            output.push_str("\n\n");
                        }
                        
                        // Use # characters for each heading level
                        let prefix = "#".repeat(level_num);
                        let styled_prefix = StyledText::new(&prefix)
                            .foreground(ThemeColors::HEADER)
                            .style(Style::Bold);
                        
                        output.push_str(&format!("{} ", styled_prefix));
                        format_stack.push(FormatType::Heading(level_num));
                    }
                    Tag::Paragraph => {
                        if !output.is_empty() && !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }
                    }
                    Tag::BlockQuote => {
                        output.push_str(&format!("{} ", StyledText::new(">")
                            .foreground(Color::Yellow)
                            .style(Style::Bold)));
                        format_stack.push(FormatType::BlockQuote);
                    }
                    Tag::CodeBlock(kind) => {
                        code_block = true;
                        output.push_str("\n");
                        
                        // Capture code language for syntax highlighting
                        if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                            code_language = lang.to_string();
                            let lang_display = if !code_language.is_empty() {
                                format!(" ({})", code_language)
                            } else {
                                String::new()
                            };
                            
                            output.push_str(&format!("{}{}\n",
                                StyledText::new("```")
                                    .foreground(Color::BrightYellow)
                                    .style(Style::Bold),
                                StyledText::new(&lang_display)
                                    .foreground(Color::BrightBlue)
                            ));
                        } else {
                            output.push_str(&format!("{}\n",
                                StyledText::new("```")
                                    .foreground(Color::BrightYellow)
                                    .style(Style::Bold)
                            ));
                        }
                    }
                    Tag::List(first_item_number) => {
                        if !output.is_empty() && !output.ends_with("\n") {
                            output.push_str("\n");
                        }
                        
                        let list_type = if let Some(number) = first_item_number {
                            ListType::Ordered(number as usize)
                        } else {
                            ListType::Unordered
                        };
                        
                        list_stack.push(list_type);
                    }
                    Tag::Item => {
                        if !output.ends_with("\n") && !output.is_empty() {
                            output.push_str("\n");
                        }
                        
                        // Calculate indent based on list nesting level
                        let indent = " ".repeat((list_stack.len() - 1) * 2);
                        
                        // Format list item based on list type
                        if let Some(list_type) = list_stack.last_mut() {
                            match list_type {
                                ListType::Unordered => {
                                    output.push_str(&format!("{}{}  ",
                                        indent,
                                        StyledText::new("•")
                                            .foreground(Color::BrightCyan)
                                            .style(Style::Bold)
                                    ));
                                }
                                ListType::Ordered(number) => {
                                    output.push_str(&format!("{}{}.  ",
                                        indent,
                                        StyledText::new(&number.to_string())
                                            .foreground(Color::BrightGreen)
                                            .style(Style::Bold)
                                    ));
                                    // Mutate the list element to increment the counter for next item
                                    if let Some(ListType::Ordered(n)) = list_stack.last_mut() {
                                        *n += 1;
                                    }
                                }
                            }
                        }
                    }
                    Tag::Emphasis => {
                        format_stack.push(FormatType::Emphasis);
                    }
                    Tag::Strong => {
                        format_stack.push(FormatType::Strong);
                    }
                    Tag::Link(_, url, _) => {
                        format_stack.push(FormatType::Link(url.to_string()));
                    }
                    Tag::Table(_) => {
                        output.push_str("\n");
                        format_stack.push(FormatType::Table);
                    }
                    Tag::TableHead => {
                        format_stack.push(FormatType::TableHeader);
                    }
                    Tag::TableRow => {
                        format_stack.push(FormatType::TableRow);
                    }
                    Tag::TableCell => {
                        format_stack.push(FormatType::TableCell);
                    }
                    Tag::Image(_, _, _) => {
                        output.push_str(&format!("{} ",
                            StyledText::new("[Image]")
                                .foreground(Color::Magenta)
                                .style(Style::Italic)
                        ));
                    }
                    _ => {}
                }
            }
            Event::End(tag) => {
                match tag {
                    Tag::Heading(..) => {
                        format_stack.pop();
                        output.push_str("\n");
                    }
                    Tag::Paragraph => {
                        pending_newlines = 2;
                    }
                    Tag::BlockQuote => {
                        format_stack.pop();
                        output.push_str("\n");
                    }
                    Tag::CodeBlock(_) => {
                        code_block = false;
                        code_language = String::new();
                        output.push_str(&format!("{}\n\n",
                            StyledText::new("```")
                                .foreground(Color::BrightYellow)
                                .style(Style::Bold)
                        ));
                    }
                    Tag::List(_) => {
                        list_stack.pop();
                        if list_stack.is_empty() {
                            output.push_str("\n");
                        }
                    }
                    Tag::Item => {
                        // No special action needed for now
                    }
                    Tag::Emphasis => {
                        format_stack.pop();
                    }
                    Tag::Strong => {
                        format_stack.pop();
                    }
                    Tag::Link(_, _, _) => {
                        if let Some(FormatType::Link(url)) = format_stack.last() {
                            output.push_str(&format!(" ({})",
                                StyledText::new(url)
                                    .foreground(Color::Blue)
                                    .style(Style::Underline)
                            ));
                        }
                        format_stack.pop();
                    }
                    Tag::Table(_) => {
                        format_stack.pop();
                        output.push_str("\n\n");
                    }
                    Tag::TableHead | Tag::TableRow | Tag::TableCell => {
                        format_stack.pop();
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                flush_newlines(&mut output, &mut pending_newlines);
                let text_str = text.to_string();
                
                // Apply formatting based on the current format context
                let styled_text = if code_block {
                    format_code(&text_str, &code_language)
                } else if let Some(format_type) = format_stack.last() {
                    match format_type {
                        FormatType::Heading(level) => {
                            StyledText::new(&text_str)
                                .foreground(ThemeColors::HEADER)
                                .style(match level {
                                    1 => Style::Bold,
                                    _ => Style::Bold,
                                })
                                .to_string()
                        }
                        FormatType::Emphasis => {
                            StyledText::new(&text_str)
                                .style(Style::Italic)
                                .to_string()
                        }
                        FormatType::Strong => {
                            StyledText::new(&text_str)
                                .style(Style::Bold)
                                .to_string()
                        }
                        FormatType::Link(_) => {
                            StyledText::new(&text_str)
                                .foreground(Color::Blue)
                                .style(Style::Underline)
                                .to_string()
                        }
                        FormatType::TableHeader => {
                            StyledText::new(&text_str)
                                .foreground(ThemeColors::TABLE_HEADER)
                                .style(Style::Bold)
                                .to_string()
                        }
                        FormatType::TableCell => {
                            text_str
                        }
                        _ => text_str,
                    }
                } else {
                    text_str
                };
                
                output.push_str(&styled_text);
            }
            Event::Code(code) => {
                flush_newlines(&mut output, &mut pending_newlines);
                // Inline code
                output.push_str(&format!("`{}`",
                    StyledText::new(&code.to_string())
                        .foreground(Color::BrightYellow)
                ));
            }
            Event::Html(html) => {
                flush_newlines(&mut output, &mut pending_newlines);
                // Just pass through HTML as-is
                output.push_str(&html.to_string());
            }
            Event::HardBreak => {
                output.push_str("\n");
            }
            Event::SoftBreak => {
                output.push(' ');
            }
            Event::Rule => {
                flush_newlines(&mut output, &mut pending_newlines);
                output.push_str("\n");
                let divider = "─".repeat(terminal_width.min(80));
                output.push_str(&StyledText::new(&divider)
                    .foreground(ThemeColors::SEPARATOR)
                    .to_string());
                output.push_str("\n\n");
            }
            Event::TaskListMarker(checked) => {
                let marker = if checked {
                    StyledText::new("[✓]")
                        .foreground(Color::Green)
                        .style(Style::Bold)
                } else {
                    StyledText::new("[ ]")
                        .foreground(Color::Yellow)
                };
                output.push_str(&format!("{} ", marker));
            }
            Event::FootnoteReference(reference) => {
                output.push_str(&format!("[{}]",
                    StyledText::new(&reference.to_string())
                        .foreground(Color::Magenta)
                ));
            }
        }
    }

    output
}

/// Helper function to format code blocks with syntax highlighting
fn format_code(code: &str, language: &str) -> String {
    // Basic code formatting - we could add more complex syntax highlighting in the future
    match language {
        "rust" => highlight_rust(code),
        _ => StyledText::new(code)
            .foreground(Color::BrightWhite)
            .to_string()
    }
}

/// Basic Rust syntax highlighting
fn highlight_rust(code: &str) -> String {
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

/// Helper to get terminal width
fn get_terminal_width() -> usize {
    terminal_size().map_or(80, |(Width(w), _)| w as usize)
}

/// Helper function to add newlines
fn flush_newlines(output: &mut String, pending: &mut usize) {
    if *pending > 0 {
        output.push_str(&"\n".repeat(*pending));
        *pending = 0;
    }
}

/// Enum to track the current formatting context
#[derive(Debug)]
enum FormatType {
    Heading(usize),
    Emphasis,
    Strong,
    Link(String),
    BlockQuote,
    Table,
    TableHeader,
    TableRow,
    TableCell,
}

/// Enum to track list types
#[derive(Debug)]
enum ListType {
    Ordered(usize),
    Unordered,
}

/// Format a markdown string for terminal output
pub fn render_markdown(markdown: &str) -> String {
    format_markdown(markdown)
}