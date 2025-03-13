use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndentationType {
    Spaces(usize),
    Tabs,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BraceStyle {
    SameLine,
    NextLine,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamingConvention {
    CamelCase,
    SnakeCase,
    PascalCase,
    KebabCase,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct LineMetrics {
    pub avg_length: f64,
    pub max_length: usize,
    pub over_limit_count: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionMetrics {
    pub avg_length: f64,
    pub max_length: usize,
    pub avg_params: f64,
    pub max_params: usize,
}

#[derive(Debug, Clone)]
pub struct StyleProfile {
    // File information
    pub file_type: String, // Renamed from language to more accurate name
    pub indentation: IndentationType,
    pub brace_style: BraceStyle,
    pub line_metrics: LineMetrics,
    pub function_metrics: FunctionMetrics,
    pub naming: HashMap<String, NamingConvention>,
    pub has_trailing_semicolons: Option<bool>,
    pub trailing_whitespace_count: usize,
    pub comment_ratio: f64,
}

impl Default for StyleProfile {
    fn default() -> Self {
        Self {
            file_type: String::new(),
            indentation: IndentationType::Unknown,
            brace_style: BraceStyle::Unknown,
            line_metrics: LineMetrics {
                avg_length: 0.0,
                max_length: 0,
                over_limit_count: 0,
            },
            function_metrics: FunctionMetrics {
                avg_length: 0.0,
                max_length: 0,
                avg_params: 0.0,
                max_params: 0,
            },
            naming: HashMap::new(),
            has_trailing_semicolons: None,
            trailing_whitespace_count: 0,
            comment_ratio: 0.0,
        }
    }
}

impl StyleProfile {
    pub fn new(language: &str) -> Self {
        Self {
            file_type: language.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct StyleInconsistency {
    pub file_path: String,
    pub line_number: Option<usize>,
    pub description: String,
    pub severity: InconsistencySeverity,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum InconsistencySeverity {
    #[default]
    Info,
    Low,
    Medium,
    High,
}

#[derive(Debug, Default)]
pub struct CodeStyleAnalysis {
    pub file_profiles: HashMap<String, StyleProfile>,
    pub global_profile: StyleProfile,
    pub inconsistencies: Vec<StyleInconsistency>,
    pub consistency_score: f64,
}

impl CodeStyleAnalysis {
    pub fn new() -> Self {
        Self {
            file_profiles: HashMap::new(),
            global_profile: StyleProfile::default(),
            inconsistencies: Vec::new(),
            consistency_score: 0.0,
        }
    }
}