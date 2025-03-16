pub struct LanguageDetector;

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageDetector {
    pub fn new() -> Self {
        LanguageDetector
    }

    pub fn detect_by_filename(&self, filename: &str) -> String {
        match filename {
            ".gitignore" => "GitConfig",
            "Dockerfile" => "Docker",
            "Makefile" => "Make",
            ".dockerignore" => "Docker",
            "LICENSE" => "License",
            ".DS_Store" => "SystemFile",
            "web.config" => "ASP.NET",
            "global.asax" => "ASP.NET",
            "Assembly.cs" | "AssemblyInfo.cs" => "C#",
            "AssemblyInfo.vb" => "VisualBasic",
            "NuGet.config" | "nuget.config" => "DotNetProject",
            _ => "Other",
        }
        .to_string()
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
            "sh" | "bash" => "Shell",
            "lock" => "LockFile",
            "sample" => "Sample",
            "cs" => "C#",
            "vb" => "VisualBasic",
            "fs" => "FSharp",
            "xaml" => "XAML",
            "cshtml" | "razor" => "Razor",
            "aspx" | "ascx" => "ASP.NET",
            "csproj" | "vbproj" | "fsproj" | "sln" => "DotNetProject",
            _ => "Other",
        }
        .to_string()
    }

    pub fn get_comment_syntax(&self, language: &str) -> (String, String, String) {
        match language {
            "Rust" => ("//".to_string(), "/*".to_string(), "*/".to_string()),
            "JavaScript" | "TypeScript" | "C" | "C++" | "Java" | "Go" | "C#" | "FSharp" => {
                ("//".to_string(), "/*".to_string(), "*/".to_string())
            }
            "VisualBasic" => ("'".to_string(), "/*".to_string(), "*/".to_string()),
            "XAML" | "Razor" | "ASP.NET" => ("".to_string(), "<!--".to_string(), "-->".to_string()),
            "Python" | "Shell" | "Make" | "Docker" => ("#".to_string(), "".to_string(), "".to_string()),
            "Ruby" => ("#".to_string(), "=begin".to_string(), "=end".to_string()),
            "HTML" | "CSS" => ("".to_string(), "<!--".to_string(), "-->".to_string()),
            "Markdown" | "YAML" | "TOML" | "JSON" | "LockFile" | "Sample" | "GitConfig" |
            "License" | "SystemFile" | "DotNetProject" => {
                ("".to_string(), "".to_string(), "".to_string())
            }
            _ => ("".to_string(), "".to_string(), "".to_string()),
        }
    }
}
