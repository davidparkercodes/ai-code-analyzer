pub struct LanguageDetector;

impl LanguageDetector {
    pub fn new() -> Self {
        LanguageDetector
    }

    pub fn detect_language(&self, extension: &str) -> String {
        match extension {
            "rs" => "Rust",
            "js" | "jsx" => "JavaScript",
            "ts" | "tsx" => "TypeScript",
            "py" => "Python",
            "java" => "Java",
            "c" | "h" => "C",
            "cpp" | "hpp" => "C++",
            "go" => "Go",
            "rb" => "Ruby",
            "php" => "PHP",
            "html" => "HTML",
            "css" => "CSS",
            "md" => "Markdown",
            "json" => "JSON",
            "yml" | "yaml" => "YAML",
            "toml" => "TOML",
            _ => "Other",
        }
        .to_string()
    }

    pub fn get_comment_syntax(&self, language: &str) -> (String, String, String) {
        match language {
            "Rust" => ("//".to_string(), "/*".to_string(), "*/".to_string()),
            "JavaScript" | "TypeScript" | "C" | "C++" | "Java" | "Go" => {
                ("//".to_string(), "/*".to_string(), "*/".to_string())
            }
            "Python" => ("#".to_string(), "\"\"\"".to_string(), "\"\"\"".to_string()),
            "Ruby" => ("#".to_string(), "=begin".to_string(), "=end".to_string()),
            "HTML" | "CSS" => ("".to_string(), "<!--".to_string(), "-->".to_string()),
            "Markdown" | "YAML" | "TOML" | "JSON" => {
                ("".to_string(), "".to_string(), "".to_string())
            }
            _ => ("".to_string(), "".to_string(), "".to_string()),
        }
    }
}
