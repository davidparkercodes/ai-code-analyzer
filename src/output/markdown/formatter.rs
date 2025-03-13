use pulldown_cmark::{Event, Parser, Tag};
use terminal_size::{Width, terminal_size};

use crate::output::style::{StyledText, Color, Style, ThemeColors};
use super::syntax_highlighter::format_code;

/// Format markdown text for terminal display
pub fn format_markdown(markdown_text: &str) -> String {
    let parser = Parser::new(markdown_text);
    let mut formatter = MarkdownFormatter::new();
    
    for event in parser {
        formatter.process_event(event);
    }

    formatter.output
}

/// Markdown formatter state and logic
struct MarkdownFormatter {
    output: String,
    format_stack: Vec<FormatType>,
    list_stack: Vec<ListType>,
    code_block: bool,
    code_language: String,
    terminal_width: usize,
    pending_newlines: usize,
}

impl MarkdownFormatter {
    fn new() -> Self {
        MarkdownFormatter {
            output: String::new(),
            format_stack: Vec::new(),
            list_stack: Vec::new(),
            code_block: false,
            code_language: String::new(),
            terminal_width: get_terminal_width(),
            pending_newlines: 0,
        }
    }
    
    fn process_event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.handle_start_tag(tag),
            Event::End(tag) => self.handle_end_tag(tag),
            Event::Text(text) => self.handle_text(text),
            Event::Code(code) => self.handle_inline_code(code),
            Event::Html(html) => self.handle_html(html),
            Event::HardBreak => self.handle_hard_break(),
            Event::SoftBreak => self.handle_soft_break(),
            Event::Rule => self.handle_horizontal_rule(),
            Event::TaskListMarker(checked) => self.handle_task_list_marker(checked),
            Event::FootnoteReference(reference) => self.handle_footnote_reference(reference),
        }
    }
    
    fn handle_start_tag(&mut self, tag: Tag) {
        flush_newlines(&mut self.output, &mut self.pending_newlines);
        
        match tag {
            Tag::Heading(level, _, _) => self.start_heading(level),
            Tag::Paragraph => self.start_paragraph(),
            Tag::BlockQuote => self.start_block_quote(),
            Tag::CodeBlock(kind) => self.start_code_block(kind),
            Tag::List(first_item_number) => self.start_list(first_item_number),
            Tag::Item => self.start_list_item(),
            Tag::Emphasis => self.format_stack.push(FormatType::Emphasis),
            Tag::Strong => self.format_stack.push(FormatType::Strong),
            Tag::Link(_, url, _) => self.format_stack.push(FormatType::Link(url.to_string())),
            Tag::Table(_) => self.start_table(),
            Tag::TableHead => self.format_stack.push(FormatType::TableHeader),
            Tag::TableRow => self.format_stack.push(FormatType::TableRow),
            Tag::TableCell => self.format_stack.push(FormatType::TableCell),
            Tag::Image(_, _, _) => self.handle_image(),
            _ => {}
        }
    }
    
    fn start_heading(&mut self, level: pulldown_cmark::HeadingLevel) {
        let level_num = level as usize;
        if !self.output.is_empty() {
            self.output.push_str("\n\n");
        }
        
        let prefix = "#".repeat(level_num);
        let styled_prefix = StyledText::new(&prefix)
            .foreground(ThemeColors::HEADER)
            .style(Style::Bold);
        
        self.output.push_str(&format!("{} ", styled_prefix));
        self.format_stack.push(FormatType::Heading(level_num));
    }
    
    fn start_paragraph(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with("\n\n") {
            self.output.push_str("\n\n");
        }
    }
    
    fn start_block_quote(&mut self) {
        self.output.push_str(&format!("{} ", StyledText::new(">")
            .foreground(Color::Yellow)
            .style(Style::Bold)));
        self.format_stack.push(FormatType::BlockQuote);
    }
    
    fn start_code_block(&mut self, kind: pulldown_cmark::CodeBlockKind) {
        self.code_block = true;
        self.output.push('\n');
        
        if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
            self.code_language = lang.to_string();
            let lang_display = if !self.code_language.is_empty() {
                format!(" ({})", self.code_language)
            } else {
                String::new()
            };
            
            self.output.push_str(&format!("{}{}\n",
                StyledText::new("```")
                    .foreground(Color::BrightYellow)
                    .style(Style::Bold),
                StyledText::new(&lang_display)
                    .foreground(Color::BrightBlue)
            ));
        } else {
            self.output.push_str(&format!("{}\n",
                StyledText::new("```")
                    .foreground(Color::BrightYellow)
                    .style(Style::Bold)
            ));
        }
    }
    
    fn start_list(&mut self, first_item_number: Option<u64>) {
        if !self.output.is_empty() {
            if !self.output.ends_with("\n") {
                self.output.push('\n');
            } else if self.output.ends_with("\n\n") {
                self.output.truncate(self.output.len() - 1);
            }
        }
        
        let list_type = if let Some(number) = first_item_number {
            ListType::Ordered(number as usize)
        } else {
            ListType::Unordered
        };
        
        self.list_stack.push(list_type);
    }
    
    fn start_list_item(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with("\n") {
            self.output.push('\n');
        }
        
        let indent = " ".repeat((self.list_stack.len() - 1) * 2);
        
        if let Some(list_type) = self.list_stack.last_mut() {
            match list_type {
                ListType::Unordered => {
                    self.output.push_str(&format!("{}{}  ",
                        indent,
                        StyledText::new("•")
                            .foreground(Color::BrightCyan)
                            .style(Style::Bold)
                    ));
                }
                ListType::Ordered(number) => {
                    self.output.push_str(&format!("{}{}.  ",
                        indent,
                        StyledText::new(&number.to_string())
                            .foreground(Color::BrightGreen)
                            .style(Style::Bold)
                    ));
                    if let Some(ListType::Ordered(n)) = self.list_stack.last_mut() {
                        *n += 1;
                    }
                }
            }
        }
    }
    
    fn start_table(&mut self) {
        self.output.push('\n');
        self.format_stack.push(FormatType::Table);
    }
    
    fn handle_image(&mut self) {
        self.output.push_str(&format!("{} ",
            StyledText::new("[Image]")
                .foreground(Color::Magenta)
                .style(Style::Italic)
        ));
    }
    
    fn handle_end_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Heading(..) => self.end_heading(),
            Tag::Paragraph => self.end_paragraph(),
            Tag::BlockQuote => self.end_block_quote(),
            Tag::CodeBlock(_) => self.end_code_block(),
            Tag::List(_) => self.end_list(),
            Tag::Item => { /* No action needed */ },
            Tag::Emphasis | Tag::Strong => { self.format_stack.pop(); },
            Tag::Link(_, _, _) => self.end_link(),
            Tag::Table(_) => self.end_table(),
            Tag::TableHead | Tag::TableRow | Tag::TableCell => { self.format_stack.pop(); },
            _ => {}
        }
    }
    
    fn end_heading(&mut self) {
        self.format_stack.pop();
        self.output.push('\n');
    }
    
    fn end_paragraph(&mut self) {
        self.pending_newlines = 2;
    }
    
    fn end_block_quote(&mut self) {
        self.format_stack.pop();
        self.output.push('\n');
    }
    
    fn end_code_block(&mut self) {
        self.code_block = false;
        self.code_language = String::new();
        self.output.push_str(&format!("{}\n\n",
            StyledText::new("```")
                .foreground(Color::BrightYellow)
                .style(Style::Bold)
        ));
    }
    
    fn end_list(&mut self) {
        self.list_stack.pop();
        if self.list_stack.is_empty() {
            self.output.push('\n');
        }
    }
    
    fn end_link(&mut self) {
        if let Some(FormatType::Link(url)) = self.format_stack.last() {
            self.output.push_str(&format!(" ({})",
                StyledText::new(url)
                    .foreground(Color::Blue)
                    .style(Style::Underline)
            ));
        }
        self.format_stack.pop();
    }
    
    fn end_table(&mut self) {
        self.format_stack.pop();
        self.output.push_str("\n\n");
    }
    
    fn handle_text(&mut self, text: pulldown_cmark::CowStr) {
        flush_newlines(&mut self.output, &mut self.pending_newlines);
        let text_str = text.to_string();
        
        let is_in_list = !self.list_stack.is_empty();
        if is_in_list && text_str.trim().is_empty() {
            return;
        }
        
        let styled_text = self.format_text(&text_str);
        self.output.push_str(&styled_text);
    }
    
    fn format_text(&self, text: &str) -> String {
        if self.code_block {
            format_code(text, &self.code_language)
        } else if let Some(format_type) = self.format_stack.last() {
            match format_type {
                FormatType::Heading(level) => {
                    StyledText::new(text)
                        .foreground(ThemeColors::HEADER)
                        .style(match level {
                            1 => Style::Bold,
                            _ => Style::Bold,
                        })
                        .to_string()
                }
                FormatType::Emphasis => {
                    StyledText::new(text)
                        .style(Style::Italic)
                        .to_string()
                }
                FormatType::Strong => {
                    StyledText::new(text)
                        .style(Style::Bold)
                        .to_string()
                }
                FormatType::Link(_) => {
                    StyledText::new(text)
                        .foreground(Color::Blue)
                        .style(Style::Underline)
                        .to_string()
                }
                FormatType::TableHeader => {
                    StyledText::new(text)
                        .foreground(ThemeColors::TABLE_HEADER)
                        .style(Style::Bold)
                        .to_string()
                }
                FormatType::TableCell => {
                    text.to_string()
                }
                _ => text.to_string(),
            }
        } else {
            text.to_string()
        }
    }
    
    fn handle_inline_code(&mut self, code: pulldown_cmark::CowStr) {
        flush_newlines(&mut self.output, &mut self.pending_newlines);
        self.output.push_str(&format!("`{}`",
            StyledText::new(code.as_ref())
                .foreground(Color::BrightYellow)
        ));
    }
    
    fn handle_html(&mut self, html: pulldown_cmark::CowStr) {
        flush_newlines(&mut self.output, &mut self.pending_newlines);
        self.output.push_str(html.as_ref());
    }
    
    fn handle_hard_break(&mut self) {
        self.output.push('\n');
    }
    
    fn handle_soft_break(&mut self) {
        self.output.push(' ');
    }
    
    fn handle_horizontal_rule(&mut self) {
        flush_newlines(&mut self.output, &mut self.pending_newlines);
        self.output.push('\n');
        let divider = "─".repeat(self.terminal_width.min(80));
        self.output.push_str(&StyledText::new(&divider)
            .foreground(ThemeColors::SEPARATOR)
            .to_string());
        self.output.push_str("\n\n");
    }
    
    fn handle_task_list_marker(&mut self, checked: bool) {
        let marker = if checked {
            StyledText::new("[✓]")
                .foreground(Color::Green)
                .style(Style::Bold)
        } else {
            StyledText::new("[ ]")
                .foreground(Color::Yellow)
        };
        self.output.push_str(&format!("{} ", marker));
    }
    
    fn handle_footnote_reference(&mut self, reference: pulldown_cmark::CowStr) {
        self.output.push_str(&format!("[{}]",
            StyledText::new(reference.as_ref())
                .foreground(Color::Magenta)
        ));
    }
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