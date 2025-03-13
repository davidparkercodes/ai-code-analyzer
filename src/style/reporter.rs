use crate::style::models::*;
use crate::output::style::*;
use std::collections::HashMap;

pub struct StyleReporter;

impl Default for StyleReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleReporter {
    pub fn new() -> Self {
        StyleReporter
    }
    
    pub fn report(&self, analysis: &CodeStyleAnalysis) {
        println!();
        print_header("Code Style Analysis:");
        println!(
            "{}",
            StyledText::new("====================").foreground(ThemeColors::SEPARATOR)
        );
        
        // Print overall consistency score
        self.print_consistency_score(analysis);
        
        // Print global style profile
        self.print_global_profile(&analysis.global_profile);
        
        // Print style inconsistencies
        self.print_inconsistencies(&analysis.inconsistencies);
        
        // Print style guide
        self.print_style_guide(&analysis.global_profile);
    }
    
    fn print_consistency_score(&self, analysis: &CodeStyleAnalysis) {
        println!();
        print_header("Style Consistency Score:");
        println!(
            "{}",
            StyledText::new("====================").foreground(ThemeColors::SEPARATOR)
        );
        
        // Calculate percentage
        let percentage = (analysis.consistency_score * 100.0).round() as i32;
        
        // Determine color based on score
        let score_color = if percentage >= 90 {
            Color::Green
        } else if percentage >= 70 {
            Color::Yellow
        } else {
            Color::Red
        };
        
        // Print the score
        println!(
            "{}: {}%",
            highlight("Overall Score"),
            StyledText::new(&format!("{}", percentage))
                .foreground(score_color)
                .style(Style::Bold)
        );
        
        println!(
            "{}: {}",
            highlight("Files Analyzed"),
            StyledText::new(&format!("{}", analysis.file_profiles.len()))
                .foreground(ThemeColors::NUMBER)
                .style(Style::Bold)
        );
        
        println!(
            "{}: {}",
            highlight("Inconsistencies Found"),
            StyledText::new(&format!("{}", analysis.inconsistencies.len()))
                .foreground(ThemeColors::NUMBER)
                .style(Style::Bold)
        );
    }
    
    fn print_global_profile(&self, profile: &StyleProfile) {
        println!();
        print_header("Detected Code Style:");
        println!(
            "{}",
            StyledText::new("===================").foreground(ThemeColors::SEPARATOR)
        );
        
        // Print indentation style
        println!(
            "{}: {}",
            highlight("Indentation"),
            StyledText::new(&self.format_indentation(&profile.indentation)).foreground(ThemeColors::LANGUAGE)
        );
        
        // Print brace style
        println!(
            "{}: {}",
            highlight("Brace Style"),
            StyledText::new(&self.format_brace_style(&profile.brace_style)).foreground(ThemeColors::LANGUAGE)
        );
        
        // Print line length metrics
        println!(
            "{}: {} (max: {})",
            highlight("Average Line Length"),
            StyledText::new(&format!("{:.1}", profile.line_metrics.avg_length)).foreground(ThemeColors::NUMBER),
            StyledText::new(&format!("{}", profile.line_metrics.max_length)).foreground(ThemeColors::NUMBER)
        );
        
        // Print function metrics
        println!(
            "{}: {} (max: {})",
            highlight("Average Function Parameters"),
            StyledText::new(&format!("{:.1}", profile.function_metrics.avg_params)).foreground(ThemeColors::NUMBER),
            StyledText::new(&format!("{}", profile.function_metrics.max_params)).foreground(ThemeColors::NUMBER)
        );
        
        // Print naming conventions
        for (category, convention) in &profile.naming {
            println!(
                "{} {}: {}",
                highlight("Naming Convention for"),
                StyledText::new(category).foreground(ThemeColors::LANGUAGE),
                StyledText::new(&self.format_naming_convention(convention)).foreground(ThemeColors::LANGUAGE)
            );
        }
        
        // Print semicolon usage
        if let Some(semicolons) = profile.has_trailing_semicolons {
            println!(
                "{}: {}",
                highlight("Trailing Semicolons"),
                StyledText::new(if semicolons { "Yes" } else { "No" }).foreground(ThemeColors::LANGUAGE)
            );
        }
        
        // Print comment ratio
        println!(
            "{}: {}%",
            highlight("Comment Ratio"),
            StyledText::new(&format!("{:.1}", profile.comment_ratio * 100.0)).foreground(ThemeColors::NUMBER)
        );
    }
    
    fn print_inconsistencies(&self, inconsistencies: &[StyleInconsistency]) {
        if inconsistencies.is_empty() {
            return;
        }
        
        println!();
        print_header("Style Inconsistencies:");
        println!(
            "{}",
            StyledText::new("====================").foreground(ThemeColors::SEPARATOR)
        );
        
        // Group inconsistencies by severity
        let mut by_severity: HashMap<InconsistencySeverity, Vec<&StyleInconsistency>> = HashMap::new();
        
        for inconsistency in inconsistencies {
            by_severity.entry(inconsistency.severity.clone()).or_default().push(inconsistency);
        }
        
        // Print high severity inconsistencies first
        let order = [
            InconsistencySeverity::High,
            InconsistencySeverity::Medium,
            InconsistencySeverity::Low,
            InconsistencySeverity::Info,
        ];
        
        for severity in &order {
            if let Some(items) = by_severity.get(severity) {
                // Print section header
                let severity_text = match severity {
                    InconsistencySeverity::High => "High Severity",
                    InconsistencySeverity::Medium => "Medium Severity",
                    InconsistencySeverity::Low => "Low Severity",
                    InconsistencySeverity::Info => "Info",
                };
                
                let severity_color = match severity {
                    InconsistencySeverity::High => Color::Red,
                    InconsistencySeverity::Medium => Color::Yellow,
                    InconsistencySeverity::Low => Color::Green,
                    InconsistencySeverity::Info => ThemeColors::LABEL,
                };
                
                println!();
                println!(
                    "{}",
                    StyledText::new(severity_text).foreground(severity_color).style(Style::Bold)
                );
                println!(
                    "{}",
                    StyledText::new(&"-".repeat(severity_text.len())).foreground(severity_color)
                );
                
                // Print inconsistencies
                for item in items {
                    let file_name = self.format_file_path(&item.file_path);
                    
                    print!(
                        "{} ",
                        StyledText::new(&file_name).foreground(ThemeColors::LANGUAGE).style(Style::Bold)
                    );
                    
                    if let Some(line) = item.line_number {
                        print!(
                            "{} ",
                            StyledText::new(&format!("line {}", line)).foreground(ThemeColors::NUMBER)
                        );
                    }
                    
                    println!(
                        "- {}",
                        StyledText::new(&item.description).foreground(ThemeColors::LABEL)
                    );
                }
            }
        }
    }
    
    fn print_style_guide(&self, profile: &StyleProfile) {
        println!();
        print_header("Suggested Style Guide:");
        println!(
            "{}",
            StyledText::new("=====================").foreground(ThemeColors::SEPARATOR)
        );
        
        println!("Based on the analyzed codebase, here is a suggested style guide:");
        println!();
        
        // Indentation
        println!(
            "{}{}",
            StyledText::new("• ").foreground(ThemeColors::NUMBER),
            highlight("Indentation: ")
        );
        match profile.indentation {
            IndentationType::Spaces(n) => {
                println!("  Use {} spaces for indentation", n);
            },
            IndentationType::Tabs => {
                println!("  Use tabs for indentation");
            },
            _ => {
                println!("  Use consistent indentation (recommendation: 4 spaces)");
            }
        }
        println!();
        
        // Brace style
        println!(
            "{}{}",
            StyledText::new("• ").foreground(ThemeColors::NUMBER),
            highlight("Braces: ")
        );
        match profile.brace_style {
            BraceStyle::SameLine => {
                println!("  Place opening braces on the same line as the declaration");
            },
            BraceStyle::NextLine => {
                println!("  Place opening braces on the next line after the declaration");
            },
            _ => {
                println!("  Use consistent brace placement");
            }
        }
        println!();
        
        // Line length
        println!(
            "{}{}",
            StyledText::new("• ").foreground(ThemeColors::NUMBER),
            highlight("Line Length: ")
        );
        println!("  Keep lines under {} characters", 100);
        println!("  Current average: {:.1} characters", profile.line_metrics.avg_length);
        println!();
        
        // Function length & parameters
        println!(
            "{}{}",
            StyledText::new("• ").foreground(ThemeColors::NUMBER),
            highlight("Functions: ")
        );
        println!("  Keep functions concise and focused on a single task");
        println!("  Limit parameter count to {} or fewer", 5);
        println!("  Current average params: {:.1}", profile.function_metrics.avg_params);
        println!();
        
        // Naming conventions
        println!(
            "{}{}",
            StyledText::new("• ").foreground(ThemeColors::NUMBER),
            highlight("Naming Conventions: ")
        );
        
        for (category, convention) in &profile.naming {
            let convention_text = match convention {
                NamingConvention::CamelCase => "camelCase",
                NamingConvention::SnakeCase => "snake_case",
                NamingConvention::PascalCase => "PascalCase",
                NamingConvention::KebabCase => "kebab-case",
                _ => "consistent naming",
            };
            
            println!("  Use {} for {}", convention_text, category);
        }
        println!();
        
        // Whitespace
        println!(
            "{}{}",
            StyledText::new("• ").foreground(ThemeColors::NUMBER),
            highlight("Whitespace: ")
        );
        println!("  Avoid trailing whitespace");
        println!("  Use blank lines to separate logical blocks of code");
        println!();
        
        // Comments
        println!(
            "{}{}",
            StyledText::new("• ").foreground(ThemeColors::NUMBER),
            highlight("Comments: ")
        );
        println!("  Write meaningful comments that explain why, not what");
        
        let comment_percentage = profile.comment_ratio * 100.0;
        if comment_percentage < 5.0 {
            println!("  Consider adding more comments (current: {:.1}%)", comment_percentage);
        } else if comment_percentage > 30.0 {
            println!("  Aim for more self-documenting code (current comment ratio: {:.1}%)", comment_percentage);
        } else {
            println!("  Current comment ratio: {:.1}%", comment_percentage);
        }
        println!();
        
        // Semicolons
        if let Some(semicolons) = profile.has_trailing_semicolons {
            println!(
                "{}{}",
                StyledText::new("• ").foreground(ThemeColors::NUMBER),
                highlight("Semicolons: ")
            );
            if semicolons {
                println!("  Use semicolons at the end of statements");
            } else {
                println!("  Omit semicolons at the end of statements (where optional)");
            }
        }
    }
    
    fn format_indentation(&self, indentation: &IndentationType) -> String {
        match indentation {
            IndentationType::Spaces(n) => format!("{} spaces", n),
            IndentationType::Tabs => "Tabs".to_string(),
            IndentationType::Mixed => "Mixed (inconsistent)".to_string(),
            IndentationType::Unknown => "Unknown".to_string(),
        }
    }
    
    fn format_brace_style(&self, style: &BraceStyle) -> String {
        match style {
            BraceStyle::SameLine => "Same line".to_string(),
            BraceStyle::NextLine => "Next line".to_string(),
            BraceStyle::Mixed => "Mixed (inconsistent)".to_string(),
            BraceStyle::Unknown => "Unknown".to_string(),
        }
    }
    
    fn format_naming_convention(&self, convention: &NamingConvention) -> String {
        match convention {
            NamingConvention::CamelCase => "camelCase".to_string(),
            NamingConvention::SnakeCase => "snake_case".to_string(),
            NamingConvention::PascalCase => "PascalCase".to_string(),
            NamingConvention::KebabCase => "kebab-case".to_string(),
            NamingConvention::Mixed => "Mixed (inconsistent)".to_string(),
            NamingConvention::Unknown => "Unknown".to_string(),
        }
    }
    
    fn format_file_path(&self, path: &str) -> String {
        // Simplify path for display
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() <= 2 {
            path.to_string()
        } else {
            format!(".../{}", parts.last().unwrap_or(&path))
        }
    }
}