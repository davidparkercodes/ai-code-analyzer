use std::collections::HashMap;
use std::fmt;

use crate::style_analyzer::pattern::{
    BracketStyle, IndentationStyle, NamingConvention, StylePattern, StyleRule,
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
        let dominant_patterns = self.get_dominant_patterns();
        
        if dominant_patterns.is_empty() {
            self.style_guide = Some("No consistent style patterns were detected in the codebase.".to_string());
            return;
        }
        
        let mut guide = String::new();
        guide.push_str("# Code Style Guide\n\n");
        guide.push_str("This style guide was automatically generated based on detected patterns in your codebase.\n\n");
        
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
        
        // Group patterns by language
        let mut lang_patterns: HashMap<String, Vec<&StylePattern>> = HashMap::new();
        
        for pattern in dominant_patterns {
            lang_patterns
                .entry(pattern.language.clone())
                .or_default()
                .push(pattern);
        }
        
        // Add language-specific style guides
        for (language, patterns) in lang_patterns {
            guide.push_str(&format!("## {} Style Guide\n\n", language));
            
            // Naming conventions
            if let Some(pattern) = patterns.iter().find(|p| {
                matches!(p.rule, StyleRule::NamingConvention(_))
            }) {
                guide.push_str("### Naming Conventions\n\n");
                
                match &pattern.rule {
                    StyleRule::NamingConvention(convention) => {
                        match convention {
                            NamingConvention::CamelCase => {
                                guide.push_str("- Use **camelCase** for variable names and function names\n");
                                guide.push_str("- Example: `someVariable`, `calculateTotal`\n\n");
                            }
                            NamingConvention::PascalCase => {
                                guide.push_str("- Use **PascalCase** for class and type names\n");
                                guide.push_str("- Example: `UserAccount`, `DatabaseConnection`\n\n");
                            }
                            NamingConvention::SnakeCase => {
                                guide.push_str("- Use **snake_case** for variable names and function names\n");
                                guide.push_str("- Example: `user_account`, `calculate_total`\n\n");
                            }
                            NamingConvention::ScreamingSnakeCase => {
                                guide.push_str("- Use **SCREAMING_SNAKE_CASE** for constants\n");
                                guide.push_str("- Example: `MAX_USERS`, `DEFAULT_TIMEOUT`\n\n");
                            }
                            NamingConvention::KebabCase => {
                                guide.push_str("- Use **kebab-case** for filenames and URLs\n");
                                guide.push_str("- Example: `user-profile.js`, `api-client.ts`\n\n");
                            }
                            NamingConvention::Mixed => {
                                guide.push_str("- Multiple naming conventions detected in the codebase\n");
                                guide.push_str("- Consider standardizing on a single convention for each type of identifier\n\n");
                                
                                if !pattern.examples.is_empty() {
                                    guide.push_str("Examples found:\n\n");
                                    for example in &pattern.examples {
                                        guide.push_str(&format!("- `{}`\n", example));
                                    }
                                    guide.push_str("\n");
                                }
                            }
                        }
                    }
                    _ => {}
                }
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
                                guide.push_str(&format!("- Use **{} spaces** for indentation\n", n));
                                guide.push_str("- Do not use tabs for indentation\n\n");
                            }
                            IndentationStyle::Tabs => {
                                guide.push_str("- Use **tabs** for indentation\n");
                                guide.push_str("- Do not use spaces for indentation\n\n");
                            }
                            IndentationStyle::Mixed => {
                                guide.push_str("- **Mixed indentation** detected (both spaces and tabs)\n");
                                guide.push_str("- Recommendation: Standardize on either spaces or tabs\n\n");
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            // Bracket style
            if let Some(pattern) = patterns.iter().find(|p| {
                matches!(p.rule, StyleRule::BracketStyle(_))
            }) {
                guide.push_str("### Bracket Style\n\n");
                
                match &pattern.rule {
                    StyleRule::BracketStyle(style) => {
                        match style {
                            BracketStyle::SameLine => {
                                guide.push_str("- Place opening braces on the **same line** as declarations\n");
                                if !pattern.examples.is_empty() {
                                    guide.push_str("- Example:\n\n```\n");
                                    guide.push_str(&pattern.examples[0]);
                                    guide.push_str("\n```\n\n");
                                }
                            }
                            BracketStyle::NewLine => {
                                guide.push_str("- Place opening braces on a **new line** after declarations\n");
                                if !pattern.examples.is_empty() {
                                    guide.push_str("- Example:\n\n```\n");
                                    guide.push_str(&pattern.examples[0]);
                                    guide.push_str("\n```\n\n");
                                }
                            }
                            BracketStyle::Mixed => {
                                guide.push_str("- **Inconsistent bracket style** detected\n");
                                guide.push_str("- Recommendation: Standardize on either same-line or new-line style\n\n");
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            // Line length
            if let Some(pattern) = patterns.iter().find(|p| {
                matches!(p.rule, StyleRule::MaxLineLength(_))
            }) {
                guide.push_str("### Line Length\n\n");
                
                match &pattern.rule {
                    StyleRule::MaxLineLength(length) => {
                        guide.push_str(&format!("- Maximum line length: **{} characters**\n", length));
                        guide.push_str("- Break long lines when they exceed this limit\n\n");
                        
                        if !pattern.examples.is_empty() {
                            guide.push_str("Examples of long lines:\n\n");
                            for example in &pattern.examples {
                                guide.push_str(&format!("> {}\n", example));
                            }
                            guide.push_str("\n");
                        }
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
                        guide.push_str(&format!("- Maximum function length: **{} lines**\n", size));
                        guide.push_str("- Keep functions concise and focused on a single responsibility\n\n");
                    }
                    _ => {}
                }
            }
            
            // File organization
            if let Some(pattern) = patterns.iter().find(|p| {
                matches!(p.rule, StyleRule::FileOrganization(_))
            }) {
                guide.push_str("### File Organization\n\n");
                
                match &pattern.rule {
                    StyleRule::FileOrganization(org) => {
                        match org {
                            crate::style_analyzer::pattern::FileOrganization::ImportsFirst => {
                                guide.push_str("- Place all imports at the top of the file\n");
                                guide.push_str("- Group imports logically (standard library, external, internal)\n\n");
                            }
                            crate::style_analyzer::pattern::FileOrganization::TypesBeforeFunctions => {
                                guide.push_str("- Define types and classes before functions\n");
                                guide.push_str("- Group related types together\n\n");
                            }
                            crate::style_analyzer::pattern::FileOrganization::FunctionsGroupedByVisibility => {
                                guide.push_str("- Group functions by visibility (public, then private)\n");
                                guide.push_str("- Keep related functions together\n\n");
                            }
                            crate::style_analyzer::pattern::FileOrganization::TestsAtEnd => {
                                guide.push_str("- Place test functions at the end of the file\n");
                                guide.push_str("- Group tests by the functionality they test\n\n");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Add general recommendations
        guide.push_str("## General Recommendations\n\n");
        guide.push_str("- Be consistent with the existing codebase style\n");
        guide.push_str("- Use meaningful and descriptive names for variables, functions, and types\n");
        guide.push_str("- Keep functions small and focused on a single responsibility\n");
        guide.push_str("- Minimize nesting levels to improve readability\n");
        guide.push_str("- Add meaningful comments for complex logic\n");
        
        self.style_guide = Some(guide);
    }

    pub fn get_style_guide(&self) -> Option<&str> {
        self.style_guide.as_deref()
    }
}

impl fmt::Display for StyleReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "== Code Style Analysis Report ==")?;
        
        // Language statistics
        writeln!(f, "\nLanguage Statistics:")?;
        for (language, count) in &self.language_stats {
            writeln!(f, "  {}: {} files", language, count)?;
        }
        
        // Dominant patterns
        let dominant_patterns = self.get_dominant_patterns();
        writeln!(f, "\nDominant Style Patterns:")?;
        
        if dominant_patterns.is_empty() {
            writeln!(f, "  No consistent style patterns detected")?;
        } else {
            for pattern in dominant_patterns {
                write!(f, "  [{}] ", pattern.language)?;
                
                match &pattern.rule {
                    StyleRule::NamingConvention(convention) => {
                        write!(f, "Naming Convention: {:?}", convention)?;
                    }
                    StyleRule::MaxLineLength(length) => {
                        write!(f, "Max Line Length: {} chars", length)?;
                    }
                    StyleRule::IndentationStyle(style) => {
                        match style {
                            IndentationStyle::Spaces(n) => write!(f, "Indentation: {} spaces", n)?,
                            IndentationStyle::Tabs => write!(f, "Indentation: Tabs")?,
                            IndentationStyle::Mixed => write!(f, "Indentation: Mixed (inconsistent)")?,
                        }
                    }
                    StyleRule::BracketStyle(style) => {
                        write!(f, "Bracket Style: {:?}", style)?;
                    }
                    StyleRule::FunctionSize(size) => {
                        write!(f, "Function Size: max {} lines", size)?;
                    }
                    StyleRule::FileOrganization(org) => {
                        write!(f, "File Organization: {:?}", org)?;
                    }
                    StyleRule::Custom(name) => {
                        write!(f, "Custom Rule: {}", name)?;
                    }
                }
                
                writeln!(f, " ({}% consistent)", (pattern.consistency * 100.0) as usize)?;
            }
        }
        
        // Suggestions for inconsistent patterns
        let inconsistent_patterns: Vec<_> = self.patterns.iter()
            .filter(|p| p.consistency < 0.7 && p.consistency > 0.0)
            .collect();
            
        if !inconsistent_patterns.is_empty() {
            writeln!(f, "\nInconsistent Style Patterns:")?;
            
            for pattern in inconsistent_patterns {
                write!(f, "  [{}] ", pattern.language)?;
                
                match &pattern.rule {
                    StyleRule::NamingConvention(NamingConvention::Mixed) => {
                        writeln!(f, "Mixed naming conventions detected ({}% consistent)",
                            (pattern.consistency * 100.0) as usize)?;
                    }
                    StyleRule::BracketStyle(BracketStyle::Mixed) => {
                        writeln!(f, "Inconsistent bracket style ({}% consistent)",
                            (pattern.consistency * 100.0) as usize)?;
                    }
                    StyleRule::IndentationStyle(IndentationStyle::Mixed) => {
                        writeln!(f, "Mixed indentation (tabs and spaces) ({}% consistent)",
                            (pattern.consistency * 100.0) as usize)?;
                    }
                    _ => {
                        writeln!(f, "{:?} ({}% consistent)",
                            pattern.rule, (pattern.consistency * 100.0) as usize)?;
                    }
                }
            }
        }
        
        // Style guide note
        writeln!(f, "\nA comprehensive style guide based on these patterns is available.")?;
        
        Ok(())
    }
}