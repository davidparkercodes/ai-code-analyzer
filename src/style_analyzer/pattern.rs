use std::collections::HashMap;

/// Represents a coding style rule that can be detected and enforced
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StyleRule {
    // Naming conventions
    NamingConvention(NamingConvention),
    
    // Structural patterns
    MaxLineLength(usize),
    IndentationStyle(IndentationStyle),
    BracketStyle(BracketStyle),
    
    // Language specific idioms
    FunctionSize(usize),
    FileOrganization(FileOrganization),
    
    // Custom rule
    Custom(String),
}

/// Represents a detected style pattern with its frequency in the codebase
#[derive(Debug, Clone)]
pub struct StylePattern {
    pub rule: StyleRule,
    pub occurrences: usize,
    pub consistency: f64, // 0.0 to 1.0 representing consistency across codebase
    pub examples: Vec<String>, // Examples of code following this pattern
    pub language: String,
}

impl StylePattern {
    pub fn new(rule: StyleRule, language: &str) -> Self {
        StylePattern {
            rule,
            occurrences: 0,
            consistency: 0.0,
            examples: Vec::new(),
            language: language.to_string(),
        }
    }
    
    pub fn add_occurrence(&mut self, example: Option<String>) {
        self.occurrences += 1;
        if let Some(ex) = example {
            if self.examples.len() < 3 && !ex.is_empty() {
                self.examples.push(ex);
            }
        }
    }
    
    pub fn update_consistency(&mut self, total_opportunities: usize) {
        if total_opportunities > 0 {
            self.consistency = self.occurrences as f64 / total_opportunities as f64;
        }
    }
}

/// Naming convention patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NamingConvention {
    CamelCase,
    PascalCase,
    SnakeCase,
    KebabCase,
    ScreamingSnakeCase,
    Mixed,
}

/// Indentation style patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IndentationStyle {
    Spaces(usize),
    Tabs,
    Mixed,
}

/// Bracket placement style
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BracketStyle {
    SameLine,
    NewLine,
    Mixed,
}

/// File organization patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileOrganization {
    ImportsFirst,
    TypesBeforeFunctions,
    FunctionsGroupedByVisibility,
    TestsAtEnd,
}

/// Collection of style patterns by language and rule
#[derive(Debug, Clone)]
pub struct StylePatternCollection {
    patterns: HashMap<String, HashMap<StyleRule, StylePattern>>,
}

impl StylePatternCollection {
    pub fn new() -> Self {
        StylePatternCollection {
            patterns: HashMap::new(),
        }
    }
    
    pub fn add_pattern(&mut self, pattern: StylePattern) {
        let language_patterns = self.patterns
            .entry(pattern.language.clone())
            .or_insert_with(HashMap::new);
        
        language_patterns.insert(pattern.rule.clone(), pattern);
    }
    
    pub fn get_patterns(&self, language: &str) -> Vec<&StylePattern> {
        if let Some(language_patterns) = self.patterns.get(language) {
            language_patterns.values().collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn get_all_patterns(&self) -> Vec<&StylePattern> {
        self.patterns.values()
            .flat_map(|language_patterns| language_patterns.values())
            .collect()
    }
}