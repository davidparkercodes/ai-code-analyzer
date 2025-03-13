use std::collections::HashMap;
use std::fmt;

use crate::style_analyzer::pattern::{
    IndentationStyle, StylePattern, StyleRule,
};

pub struct StyleReport {
    patterns: Vec<StylePattern>,
    language_stats: HashMap<String, usize>,
    style_guide: Option<String>,
}

impl StyleReport {
    pub fn new() -> Self {
        StyleReport {
            patterns: Vec::new(),
            language_stats: HashMap::new(),
            style_guide: None,
        }
    }

    pub fn add_pattern(&mut self, pattern: StylePattern) {
        self.patterns.push(pattern);
    }

    pub fn add_language_stats(&mut self, language: &str, file_count: usize) {
        self.language_stats.insert(language.to_string(), file_count);
    }

    // These methods are removed as they're currently unused
    // If needed in the future, uncomment and implement

    // Removed unused method that was causing dead code warning

    pub fn generate_style_guide(&mut self) {
        let mut guide = String::new();
        guide.push_str("# Code Metrics Analysis\n\n");
        guide.push_str("This analysis was automatically generated based on measured metrics in your codebase.\n\n");
        
        // Add language statistics
        guide.push_str("## Codebase Statistics\n\n");
        if !self.language_stats.is_empty() {
            // First pass to determine column widths
            let mut lang_width = "Language".len();
            let mut count_width = "Number of Files".len();
            
            let mut languages: Vec<_> = self.language_stats.iter().collect();
            languages.sort_by(|a, b| b.1.cmp(a.1));
            
            // Calculate required column widths
            for (language, count) in &languages {
                lang_width = std::cmp::max(lang_width, language.len());
                count_width = std::cmp::max(count_width, count.to_string().len());
            }
            
            // Create properly padded table headers
            guide.push_str(&format!("| {:<lang_width$} | {:<count_width$} |\n", "Language", "Number of Files"));
            guide.push_str(&format!("| {:-<lang_width$} | {:-<count_width$} |\n", "", ""));
            
            // Add rows with proper padding
            for (language, count) in languages {
                guide.push_str(&format!("| {:<lang_width$} | {:<count_width$} |\n", language, count));
            }
            guide.push_str("\n");
        }
        
        // Group patterns by language for organized output
        let mut lang_patterns: HashMap<String, Vec<&StylePattern>> = HashMap::new();
        
        // Add patterns to language groups
        for pattern in &self.patterns {
            // Filter out unrealistic metrics
            if let StyleRule::MaxLineLength(length) = pattern.rule {
                if length < 20 {
                    continue; // Skip unrealistically short line lengths
                }
            }
            
            lang_patterns
                .entry(pattern.language.clone())
                .or_default()
                .push(pattern);
        }
        
        // Create ordered list of languages by file count (highest to lowest)
        let mut languages: Vec<(&String, &usize)> = self.language_stats.iter().collect();
        languages.sort_by(|a, b| b.1.cmp(a.1));
        
        let lang_order: Vec<String> = languages.iter()
            .map(|(lang, _)| (*lang).clone())
            .collect();
        
        // Generate language-specific metrics in order of file count
        for lang in lang_order {
            if let Some(patterns) = lang_patterns.get(&lang) {
                guide.push_str(&format!("## {} Metrics\n\n", lang));
                
                // Line length metrics
                let mut max_line_length = None;
                let mut avg_line_length = None;
                
                for pattern in patterns {
                    match &pattern.rule {
                        StyleRule::MaxLineLength(length) => {
                            if let Some(current_max) = max_line_length {
                                if length > current_max {
                                    max_line_length = Some(length);
                                }
                            } else {
                                max_line_length = Some(length);
                            }
                        },
                        StyleRule::AvgLineLength(length) => {
                            avg_line_length = Some(length);
                        },
                        _ => {}
                    }
                }
                
                if max_line_length.is_some() || avg_line_length.is_some() {
                    guide.push_str("### Line Length\n\n");
                    
                    if let Some(length) = max_line_length {
                        guide.push_str(&format!("- Maximum line length: **{} characters**\n", length));
                    }
                    
                    if let Some(length) = avg_line_length {
                        guide.push_str(&format!("- Average line length: **{} characters**\n", length));
                    }
                    
                    guide.push_str("\n");
                }
            
                // Indentation 
                if let Some(pattern) = patterns.iter().find(|p| {
                    matches!(p.rule, StyleRule::IndentationStyle(_))
                }) {
                    guide.push_str("### Indentation\n\n");
                    
                    match &pattern.rule {
                        StyleRule::IndentationStyle(style) => {
                            match style {
                                IndentationStyle::Spaces(n) => {
                                    guide.push_str(&format!("- **{} spaces** indentation detected ({}% of files)\n", 
                                        n, (pattern.consistency * 100.0) as usize));
                                }
                                IndentationStyle::Tabs => {
                                    guide.push_str(&format!("- **Tab** indentation detected ({}% of files)\n", 
                                        (pattern.consistency * 100.0) as usize));
                                }
                                IndentationStyle::Mixed => {
                                    guide.push_str("- **Mixed indentation** detected (both spaces and tabs)\n");
                                }
                            }
                            guide.push_str("\n");
                        }
                        _ => {}
                    }
                }
                
                // Naming conventions section removed as it needs more refined analysis by identifier type
            
                // Function size
                if let Some(pattern) = patterns.iter().find(|p| {
                    matches!(p.rule, StyleRule::FunctionSize(_))
                }) {
                    guide.push_str("### Function Size\n\n");
                    
                    match &pattern.rule {
                        StyleRule::FunctionSize(size) => {
                            guide.push_str(&format!("- Average function length: **{} lines**\n", size));
                            
                            // Calculate percentile distribution if available
                            if !pattern.examples.is_empty() {
                                guide.push_str("\nFunction size examples:\n\n");
                                for example in &pattern.examples {
                                    guide.push_str(&format!("- {}\n", example));
                                }
                            }
                            guide.push_str("\n");
                        }
                        _ => {}
                    }
                }
                
                // Comment density
                if let Some(pattern) = patterns.iter().find(|p| {
                    matches!(p.rule, StyleRule::CommentDensity(_))
                }) {
                    guide.push_str("### Comment Density\n\n");
                    
                    match &pattern.rule {
                        StyleRule::CommentDensity(density) => {
                            guide.push_str(&format!("- Comment-to-code ratio: **{}%**\n", density));
                            guide.push_str(&format!("- Approximately 1 comment line per {} lines of code\n\n", 
                                if *density > 0 { 100 / *density } else { 0 }));
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Add general insights
        guide.push_str("## Metrics Insights\n\n");
        guide.push_str("- Line length: Most style guides recommend 80-120 characters maximum\n");
        guide.push_str("- Indentation: Consistent indentation improves readability\n");
        guide.push_str("- Function size: Smaller functions (under 20-30 lines) are generally more maintainable\n");
        guide.push_str("- Comment density: Aim for self-documenting code with targeted comments for complex logic\n");
        // Removed naming conventions insight as we're not analyzing them in detail yet
        
        self.style_guide = Some(guide);
    }
    
    // No longer needed - we're focusing only on concrete metrics

    pub fn get_style_guide(&self) -> Option<&str> {
        self.style_guide.as_deref()
    }
}

impl fmt::Display for StyleReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "== Code Metrics Analysis Report ==")?;
        
        // Language statistics
        writeln!(f, "\nLanguage Statistics:")?;
        
        // Sort languages by file count (highest to lowest)
        let mut languages: Vec<(&String, &usize)> = self.language_stats.iter().collect();
        languages.sort_by(|a, b| b.1.cmp(a.1));
        
        for (language, count) in &languages {
            writeln!(f, "  {}: {} files", language, count)?;
        }
        
        // Group patterns by language for organized display
        let mut lang_patterns: HashMap<String, Vec<&StylePattern>> = HashMap::new();
        
        // Skip unrealistic metrics (like very short line lengths)
        for pattern in &self.patterns {
            // Filter out unrealistic metrics
            if let StyleRule::MaxLineLength(length) = pattern.rule {
                if length < 20 {
                    continue; // Skip unrealistically short line lengths
                }
            }
            
            lang_patterns
                .entry(pattern.language.clone())
                .or_default()
                .push(pattern);
        }
        
        // Create ordered list of languages by file count
        let lang_order: Vec<String> = languages.iter()
            .map(|(lang, _)| (*lang).clone())
            .collect();
            
        // Display metrics by language in order of file count
        for lang in &lang_order {
            if let Some(patterns) = lang_patterns.get(lang) {
                writeln!(f, "\n{} Metrics:", lang)?;
                
                // Line length metrics - show only max and average
                let mut max_length = 0;
                let mut total_avg_length = 0;
                let mut avg_count = 0;
            
                // Find the maximum line length and average the avg line lengths
                for pattern in patterns.iter() {
                    match &pattern.rule {
                        StyleRule::MaxLineLength(length) => {
                            if *length > max_length {
                                max_length = *length;
                            }
                        }
                        StyleRule::AvgLineLength(length) => {
                            total_avg_length += *length;
                            avg_count += 1;
                        }
                        _ => {}
                    }
                }
                
                // Display aggregated metrics
                if max_length > 0 {
                    writeln!(f, "  Max Line Length: {} chars", max_length)?;
                }
                
                if avg_count > 0 {
                    let avg_length = total_avg_length / avg_count;
                    writeln!(f, "  Avg Line Length: {} chars", avg_length)?;
                }
            
                // Indentation
                if let Some(pattern) = patterns.iter().find(|p| matches!(p.rule, StyleRule::IndentationStyle(_))) {
                    match &pattern.rule {
                        StyleRule::IndentationStyle(style) => {
                            match style {
                                IndentationStyle::Spaces(n) => {
                                    writeln!(f, "  Indentation: {} spaces ({}% of files)", 
                                        n, (pattern.consistency * 100.0) as usize)?;
                                }
                                IndentationStyle::Tabs => {
                                    writeln!(f, "  Indentation: Tabs ({}% of files)", 
                                        (pattern.consistency * 100.0) as usize)?;
                                }
                                IndentationStyle::Mixed => {
                                    writeln!(f, "  Indentation: Mixed (tabs and spaces)")?;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                
                // Function size
                if let Some(pattern) = patterns.iter().find(|p| matches!(p.rule, StyleRule::FunctionSize(_))) {
                    match &pattern.rule {
                        StyleRule::FunctionSize(size) => {
                            writeln!(f, "  Avg Function Size: {} lines", size)?;
                        }
                        _ => {}
                    }
                }
                
                // Comment density
                if let Some(pattern) = patterns.iter().find(|p| matches!(p.rule, StyleRule::CommentDensity(_))) {
                    match &pattern.rule {
                        StyleRule::CommentDensity(density) => {
                            writeln!(f, "  Comment Density: {}% (1 comment per {} code lines)", 
                                density, if *density > 0 { 100 / *density } else { 0 })?;
                        }
                        _ => {}
                    }
                }
                
                // Naming conventions section removed as it needs more refined analysis by identifier type
            }
        }
        
        // Note about metrics report
        writeln!(f, "\nA comprehensive metrics report is available.")?;
        
        Ok(())
    }
}