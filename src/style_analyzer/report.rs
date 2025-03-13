use std::collections::HashMap;
use std::fmt;

use crate::style_analyzer::pattern::{
    IndentationStyle, NamingConvention, StylePattern, StyleRule,
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

    pub fn get_dominant_patterns(&self) -> Vec<&StylePattern> {
        let mut result = Vec::new();
        let mut rule_map: HashMap<String, Vec<&StylePattern>> = HashMap::new();

        // Group patterns by rule type and language
        for pattern in &self.patterns {
            // Special handling for unrealistic line length patterns (too short to be real)
            if let StyleRule::MaxLineLength(length) = pattern.rule {
                if length < 20 {
                    // Skip very short line length patterns as they're likely noise
                    continue;
                }
            }
            
            let rule_key = format!(
                "{:?}_{}",
                pattern.rule,
                pattern.language
            );
            rule_map.entry(rule_key).or_default().push(pattern);
        }

        // For each rule type, select the pattern with highest consistency
        for patterns in rule_map.values() {
            if let Some(dominant) = patterns.iter().max_by(|a, b| {
                a.consistency.partial_cmp(&b.consistency).unwrap()
            }) {
                if dominant.consistency >= 0.7 {
                    // Only include patterns with at least 70% consistency
                    result.push(*dominant);
                }
            }
        }

        result
    }

    pub fn generate_style_guide(&mut self) {
        let mut guide = String::new();
        guide.push_str("# Code Metrics Analysis\n\n");
        guide.push_str("This analysis was automatically generated based on measured metrics in your codebase.\n\n");
        
        // Add language statistics
        guide.push_str("## Codebase Statistics\n\n");
        if !self.language_stats.is_empty() {
            guide.push_str("| Language | Number of Files |\n");
            guide.push_str("|----------|----------------|\n");
            
            let mut languages: Vec<_> = self.language_stats.iter().collect();
            languages.sort_by(|a, b| b.1.cmp(a.1));
            
            for (language, count) in languages {
                guide.push_str(&format!("| {} | {} |\n", language, count));
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
        
        // Generate language-specific metrics
        for (language, patterns) in lang_patterns {
            guide.push_str(&format!("## {} Metrics\n\n", language));
            
            // Line length metrics
            let mut max_line_length = None;
            let mut avg_line_length = None;
            
            for pattern in &patterns {
                match pattern.rule {
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
            
            // Naming conventions
            if let Some(pattern) = patterns.iter().find(|p| {
                matches!(p.rule, StyleRule::NamingConvention(_))
            }) {
                guide.push_str("### Naming Conventions\n\n");
                
                match &pattern.rule {
                    StyleRule::NamingConvention(convention) => {
                        match convention {
                            NamingConvention::CamelCase => {
                                guide.push_str(&format!("- **camelCase** naming convention detected ({}% of identifiers)\n", 
                                    (pattern.consistency * 100.0) as usize));
                            }
                            NamingConvention::PascalCase => {
                                guide.push_str(&format!("- **PascalCase** naming convention detected ({}% of identifiers)\n", 
                                    (pattern.consistency * 100.0) as usize));
                            }
                            NamingConvention::SnakeCase => {
                                guide.push_str(&format!("- **snake_case** naming convention detected ({}% of identifiers)\n", 
                                    (pattern.consistency * 100.0) as usize));
                            }
                            NamingConvention::ScreamingSnakeCase => {
                                guide.push_str(&format!("- **SCREAMING_SNAKE_CASE** naming convention detected ({}% of identifiers)\n", 
                                    (pattern.consistency * 100.0) as usize));
                            }
                            NamingConvention::Mixed => {
                                guide.push_str("- **Mixed naming conventions** detected in the codebase\n");
                                
                                if !pattern.examples.is_empty() {
                                    guide.push_str("\nExamples of identifiers:\n\n");
                                    for example in &pattern.examples {
                                        guide.push_str(&format!("- `{}`\n", example));
                                    }
                                }
                            }
                        }
                        guide.push_str("\n");
                    }
                    _ => {}
                }
            }
            
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
        
        // Add general insights
        guide.push_str("## Metrics Insights\n\n");
        guide.push_str("- Line length: Most style guides recommend 80-120 characters maximum\n");
        guide.push_str("- Indentation: Consistent indentation improves readability\n");
        guide.push_str("- Function size: Smaller functions (under 20-30 lines) are generally more maintainable\n");
        guide.push_str("- Comment density: Aim for self-documenting code with targeted comments for complex logic\n");
        guide.push_str("- Naming conventions: Consistent naming styles improve code readability\n");
        
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
        for (language, count) in &self.language_stats {
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
        
        // Display metrics by language
        for (language, patterns) in &lang_patterns {
            writeln!(f, "\n{} Metrics:", language)?;
            
            // Line length metrics
            let line_length_patterns: Vec<_> = patterns.iter()
                .filter(|p| matches!(p.rule, StyleRule::MaxLineLength(_) | StyleRule::AvgLineLength(_)))
                .collect();
                
            if !line_length_patterns.is_empty() {
                for pattern in line_length_patterns {
                    match &pattern.rule {
                        StyleRule::MaxLineLength(length) => {
                            writeln!(f, "  Max Line Length: {} chars", length)?;
                        }
                        StyleRule::AvgLineLength(length) => {
                            writeln!(f, "  Avg Line Length: {} chars", length)?;
                        }
                        _ => {}
                    }
                }
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
            
            // Naming conventions
            if let Some(pattern) = patterns.iter().find(|p| matches!(p.rule, StyleRule::NamingConvention(_))) {
                match &pattern.rule {
                    StyleRule::NamingConvention(convention) => {
                        writeln!(f, "  Naming Convention: {:?} ({}% consistent)", 
                            convention, (pattern.consistency * 100.0) as usize)?;
                    }
                    _ => {}
                }
            }
        }
        
        // Note about metrics report
        writeln!(f, "\nA comprehensive metrics report is available.")?;
        
        Ok(())
    }
}