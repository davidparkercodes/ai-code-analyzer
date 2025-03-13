use crate::dependency::dependency_graph::DependencyGraph;
use crate::metrics::language::LanguageDetector;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct DependencyAnalyzer {
    language_detector: LanguageDetector,
    supported_languages: HashMap<String, Vec<String>>,
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyAnalyzer {
    pub fn new() -> Self {
        let mut supported_languages = HashMap::new();

        supported_languages.insert(
            "Rust".to_string(),
            vec![
                "use".to_string(),
                "mod".to_string(),
                "extern crate".to_string(),
            ],
        );

        supported_languages.insert(
            "JavaScript".to_string(),
            vec!["import".to_string(), "require".to_string()],
        );

        supported_languages.insert(
            "TypeScript".to_string(),
            vec!["import".to_string(), "require".to_string()],
        );

        supported_languages.insert(
            "Python".to_string(),
            vec!["import".to_string(), "from".to_string()],
        );

        DependencyAnalyzer {
            language_detector: LanguageDetector::new(),
            supported_languages,
        }
    }

    pub fn analyze_dependencies<P: AsRef<Path>>(
        &self,
        dir_path: P,
    ) -> Result<DependencyGraph, String> {
        let path = dir_path.as_ref();

        if !path.exists() {
            return Err(format!("Path '{}' does not exist", path.display()));
        }

        if !path.is_dir() {
            return Err(format!("Path '{}' is not a directory", path.display()));
        }

        let mut graph = DependencyGraph::new();

        let walker = WalkBuilder::new(path)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .filter_entry(|e| {
                let path_str = e.path().to_string_lossy();
                let file_name = e
                    .path()
                    .file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default();
                // Exclude git files, lock files, config files, and system files
                let is_excluded_system_file = path_str.contains("/.git/")
                    || path_str.ends_with(".lock")
                    || path_str.ends_with(".gitignore")
                    || file_name == ".DS_Store";

                // Exclude test files and directories
                let is_test_file = path_str.contains("/test")
                    || path_str.contains("/tests/")
                    || path_str.contains("_test.")
                    || path_str.ends_with("_test.rs")
                    || path_str.ends_with("_tests.rs")
                    || path_str.ends_with("Test.java")
                    || path_str.ends_with(".test.js")
                    || path_str.ends_with(".test.ts")
                    || path_str.ends_with("_spec.js")
                    || path_str.ends_with("_spec.ts")
                    || path_str.ends_with("_test.py")
                    || path_str.ends_with("test_")
                    || file_name == "test";

                // Include only if not excluded
                !is_excluded_system_file && !is_test_file
            })
            .build();

        for result in walker {
            let entry = match result {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let path = entry.path();

            if path.is_dir() {
                continue;
            }

            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

            let language = if extension.is_empty() && !file_name.is_empty() {
                self.language_detector.detect_by_filename(file_name)
            } else {
                self.language_detector.detect_language(extension)
            };

            if self.supported_languages.contains_key(&language) {
                if let Some(dependencies) = self.extract_dependencies(path, &language) {
                    let normalized_path = self.normalize_path(path);
                    graph.add_node(&normalized_path);

                    for dependency in dependencies {
                        let normalized_dependency =
                            self.resolve_dependency(&normalized_path, &dependency);
                        graph.add_node(&normalized_dependency);
                        graph.add_edge(&normalized_path, &normalized_dependency);
                    }
                }
            }
        }

        Ok(graph)
    }

    fn extract_dependencies(&self, file_path: &Path, language: &str) -> Option<Vec<String>> {
        let content = fs::read_to_string(file_path).ok()?;
        let import_patterns = self.supported_languages.get(language)?;

        let mut dependencies = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            for pattern in import_patterns {
                match language {
                    "Rust" => {
                        if pattern == "use" && trimmed.starts_with("use ") {
                            let parts: Vec<&str> =
                                trimmed.split(&[';', ' ', '{', '}', ':'][..]).collect();
                            if parts.len() > 1 {
                                let module = parts[1].trim();
                                if !module.is_empty()
                                    && !module.starts_with("crate::")
                                    && !module.starts_with("self::")
                                    && !module.starts_with("std::")
                                {
                                    dependencies.push(module.to_string());
                                }
                            }
                        } else if pattern == "mod"
                            && trimmed.starts_with("mod ")
                            && !trimmed.contains('{')
                        {
                            let parts: Vec<&str> = trimmed.split(&[';', ' '][..]).collect();
                            if parts.len() > 1 {
                                let module = parts[1].trim();
                                if !module.is_empty() {
                                    dependencies.push(module.to_string());
                                }
                            }
                        }
                    }
                    "JavaScript" | "TypeScript" => {
                        if pattern == "import" && trimmed.starts_with("import ") {
                            if let Some(from_index) = trimmed.find(" from ") {
                                if let Some(quote_start) =
                                    trimmed[from_index + 6..].find(&['\"', '\''][..])
                                {
                                    if let Some(quote_end) = trimmed
                                        [(from_index + 6 + quote_start + 1)..]
                                        .find(&['\"', '\''][..])
                                    {
                                        let module = &trimmed[(from_index + 6 + quote_start + 1)
                                            ..(from_index + 6 + quote_start + 1 + quote_end)];
                                        if !module.is_empty() && !module.starts_with('@') {
                                            dependencies.push(module.to_string());
                                        }
                                    }
                                }
                            }
                        } else if pattern == "require" && trimmed.contains("require(") {
                            let parts: Vec<&str> = trimmed.split("require(").collect();
                            if parts.len() > 1 {
                                let quote_parts: Vec<&str> =
                                    parts[1].split(&['\"', '\''][..]).collect();
                                if quote_parts.len() > 1 {
                                    let module = quote_parts[1].trim();
                                    if !module.is_empty() && !module.starts_with('@') {
                                        dependencies.push(module.to_string());
                                    }
                                }
                            }
                        }
                    }
                    "Python" => {
                        if pattern == "import" && trimmed.starts_with("import ") {
                            let parts: Vec<&str> = trimmed.split(&[' ', ','][..]).collect();
                            if parts.len() > 1 {
                                let module = parts[1].trim();
                                if !module.is_empty() && module != "as" {
                                    dependencies.push(module.to_string());
                                }
                            }
                        } else if pattern == "from" && trimmed.starts_with("from ") {
                            if let Some(import_index) = trimmed.find(" import ") {
                                let module = trimmed[5..import_index].trim();
                                if !module.is_empty() && module != "." && module != ".." {
                                    dependencies.push(module.to_string());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Some(dependencies)
    }

    fn normalize_path(&self, path: &Path) -> String {
        path.to_string_lossy().to_string()
    }

    fn resolve_dependency(&self, source_path: &str, dependency: &str) -> String {
        // Filter out std, crate and other special module references
        if dependency.starts_with("std")
            || dependency.contains("std::")
            || dependency.starts_with("crate")
            || dependency.contains("crate::")
        {
            return dependency.to_string();
        }

        let source_dir = Path::new(source_path).parent().unwrap_or(Path::new(""));
        let dependency_path = source_dir.join(dependency);

        dependency_path.to_string_lossy().to_string()
    }
}
