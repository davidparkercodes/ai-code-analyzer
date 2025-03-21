use crate::cache::AnalysisCache;
use crate::dependency::dependency_graph::DependencyGraph;
use crate::metrics::language::LanguageDetector;
use crate::util::error::{AppError, AppResult};
use crate::util::file_filter::FileFilter;
use crate::util::parallel::ParallelProcessing;
use ignore::{DirEntry, WalkBuilder};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct DependencyAnalyzer {
    language_detector: LanguageDetector,
    supported_languages: HashMap<String, Vec<String>>,
    cache: Arc<AnalysisCache>,
    parallel: bool,
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallelProcessing for DependencyAnalyzer {
    fn enable_parallel_processing(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }
    
    fn with_parallel(self, parallel: bool) -> Self {
        self.enable_parallel_processing(parallel)
    }
    
    fn is_parallel(&self) -> bool {
        self.parallel
    }
}

impl DependencyAnalyzer {
    pub fn new() -> Self {
        let mut supported_languages = HashMap::new();
        
        supported_languages.insert(
            "Rust".to_string(), 
            vec!["use".to_string(), "mod".to_string(), "extern crate".to_string()]
        );
        
        supported_languages.insert(
            "JavaScript".to_string(), 
            vec!["import".to_string(), "require".to_string()]
        );
        
        supported_languages.insert(
            "TypeScript".to_string(), 
            vec!["import".to_string(), "require".to_string()]
        );
        
        supported_languages.insert(
            "Python".to_string(), 
            vec!["import".to_string(), "from".to_string()]
        );
        
        DependencyAnalyzer {
            language_detector: LanguageDetector::new(),
            supported_languages,
            cache: Arc::new(AnalysisCache::new()),
            parallel: true,
        }
    }
    
    pub fn with_cache(cache: Arc<AnalysisCache>) -> Self {
        let mut analyzer = Self::new();
        analyzer.cache = cache;
        analyzer
    }
    
    pub fn analyze_dependencies<P: AsRef<Path>>(&self, dir_path: P) -> AppResult<DependencyGraph> {
        let path = dir_path.as_ref();
        
        if !path.exists() {
            return Err(AppError::Analysis(format!("Path '{}' does not exist", path.display())));
        }
        
        if !path.is_dir() {
            return Err(AppError::Analysis(format!("Path '{}' is not a directory", path.display())));
        }
        
        let graph = Arc::new(Mutex::new(DependencyGraph::new()));
        
        let walker = WalkBuilder::new(path)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .filter_entry(|e| {
                !FileFilter::should_exclude(e.path()) && !FileFilter::is_test_file(e.path())
            })
            .build();
            
        let entries: Vec<DirEntry> = walker
            .filter_map(|result| {
                match result {
                    Ok(entry) => {
                        if !entry.path().is_dir() {
                            Some(entry) 
                        } else {
                            None
                        }
                    },
                    Err(_) => None,
                }
            })
            .collect();
            
        let process_entry = |entry: &DirEntry| {
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            
            if let Some(cached_deps) = self.cache.get_dependencies(&path_str) {
                let normalized_path = self.normalize_path(path);
                let mut graph_guard = graph.lock().unwrap();
                graph_guard.add_node(&normalized_path);
                
                for dependency in cached_deps {
                    let normalized_dependency = self.resolve_dependency(&normalized_path, &dependency);
                    graph_guard.add_node(&normalized_dependency);
                    graph_guard.add_edge(&normalized_path, &normalized_dependency);
                }
                return;
            }
            
            let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            
            let language = if let Some(cached_lang) = self.cache.get_language(&path_str) {
                cached_lang
            } else {
                let detected_lang = if extension.is_empty() && !file_name.is_empty() {
                    self.language_detector.detect_by_filename(file_name)
                } else {
                    self.language_detector.detect_language(extension)
                };
                self.cache.cache_language(&path_str, detected_lang.clone());
                detected_lang
            };
            
            if self.supported_languages.contains_key(&language) {
                if let Some(dependencies) = self.extract_dependencies(path, &language) {
                    let normalized_path = self.normalize_path(path);
                    
                    self.cache.cache_dependencies(&path_str, dependencies.clone());
                    
                    let mut graph_guard = graph.lock().unwrap();
                    graph_guard.add_node(&normalized_path);
                    
                    for dependency in dependencies {
                        let normalized_dependency = self.resolve_dependency(&normalized_path, &dependency);
                        graph_guard.add_node(&normalized_dependency);
                        graph_guard.add_edge(&normalized_path, &normalized_dependency);
                    }
                }
            }
        };
        
        if self.parallel {
            entries.par_iter().for_each(|entry| {
                process_entry(entry);
            });
        } else {
            entries.iter().for_each(|entry| {
                process_entry(entry);
            });
        }
        
        self.cache.purge_stale_entries();
        
        let final_graph = match graph.lock() {
            Ok(guard) => guard.clone(),
            Err(e) => return Err(AppError::Analysis(format!("Error accessing dependency graph: {}", e)))
        };
        
        Ok(final_graph)
    }
    
    fn extract_dependencies(&self, file_path: &Path, language: &str) -> Option<Vec<String>> {
        let path_str = file_path.to_string_lossy().to_string();
        
        let content = if let Some(cached_content) = self.cache.get_file_content(&path_str) {
            cached_content
        } else if let Ok(file_content) = fs::read_to_string(file_path) {
            self.cache.cache_file_content(&path_str, file_content.clone());
            file_content
        } else {
            return None;
        };
        
        let import_patterns = self.supported_languages.get(language)?;
        
        match language {
            "Rust" => self.extract_rust_dependencies(&content, import_patterns),
            "JavaScript" | "TypeScript" => self.extract_js_dependencies(&content, import_patterns),
            "Python" => self.extract_python_dependencies(&content, import_patterns),
            _ => Some(Vec::new())
        }
    }
    
    fn extract_rust_dependencies(&self, content: &str, import_patterns: &[String]) -> Option<Vec<String>> {
        let mut dependencies = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            for pattern in import_patterns {
                if pattern == "use" && trimmed.starts_with("use ") {
                    self.extract_rust_use_dependency(trimmed, &mut dependencies);
                } else if pattern == "mod" && trimmed.starts_with("mod ") && !trimmed.contains('{') {
                    self.extract_rust_mod_dependency(trimmed, &mut dependencies);
                }
            }
        }
        
        Some(dependencies)
    }
    
    fn extract_rust_use_dependency(&self, line: &str, dependencies: &mut Vec<String>) {
        let parts: Vec<&str> = line.split(&[';', ' ', '{', '}', ':'][..]).collect();
        if parts.len() > 1 {
            let module = parts[1].trim();
            if !module.is_empty() && !module.starts_with("crate::") && 
               !module.starts_with("self::") && !module.starts_with("std::") {
                dependencies.push(module.to_string());
            }
        }
    }
    
    fn extract_rust_mod_dependency(&self, line: &str, dependencies: &mut Vec<String>) {
        let parts: Vec<&str> = line.split(&[';', ' '][..]).collect();
        if parts.len() > 1 {
            let module = parts[1].trim();
            if !module.is_empty() {
                dependencies.push(module.to_string());
            }
        }
    }
    
    fn extract_js_dependencies(&self, content: &str, import_patterns: &[String]) -> Option<Vec<String>> {
        let mut dependencies = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            for pattern in import_patterns {
                if pattern == "import" && trimmed.starts_with("import ") {
                    self.extract_js_import_dependency(trimmed, &mut dependencies);
                } else if pattern == "require" && trimmed.contains("require(") {
                    self.extract_js_require_dependency(trimmed, &mut dependencies);
                }
            }
        }
        
        Some(dependencies)
    }
    
    fn extract_js_import_dependency(&self, line: &str, dependencies: &mut Vec<String>) {
        if let Some(from_index) = line.find(" from ") {
            if let Some(quote_start) = line[from_index + 6..].find(&['\"', '\''][..]) {
                if let Some(quote_end) = line[(from_index + 6 + quote_start + 1)..].find(&['\"', '\''][..]) {
                    let module = &line[(from_index + 6 + quote_start + 1)..(from_index + 6 + quote_start + 1 + quote_end)];
                    if !module.is_empty() && !module.starts_with('@') {
                        dependencies.push(module.to_string());
                    }
                }
            }
        }
    }
    
    fn extract_js_require_dependency(&self, line: &str, dependencies: &mut Vec<String>) {
        let parts: Vec<&str> = line.split("require(").collect();
        if parts.len() > 1 {
            let quote_parts: Vec<&str> = parts[1].split(&['\"', '\''][..]).collect();
            if quote_parts.len() > 1 {
                let module = quote_parts[1].trim();
                if !module.is_empty() && !module.starts_with('@') {
                    dependencies.push(module.to_string());
                }
            }
        }
    }
    
    fn extract_python_dependencies(&self, content: &str, import_patterns: &[String]) -> Option<Vec<String>> {
        let mut dependencies = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            for pattern in import_patterns {
                if pattern == "import" && trimmed.starts_with("import ") {
                    self.extract_python_import_dependency(trimmed, &mut dependencies);
                } else if pattern == "from" && trimmed.starts_with("from ") {
                    self.extract_python_from_dependency(trimmed, &mut dependencies);
                }
            }
        }
        
        Some(dependencies)
    }
    
    fn extract_python_import_dependency(&self, line: &str, dependencies: &mut Vec<String>) {
        let parts: Vec<&str> = line.split(&[' ', ','][..]).collect();
        if parts.len() > 1 {
            let module = parts[1].trim();
            if !module.is_empty() && module != "as" {
                dependencies.push(module.to_string());
            }
        }
    }
    
    fn extract_python_from_dependency(&self, line: &str, dependencies: &mut Vec<String>) {
        if let Some(import_index) = line.find(" import ") {
            let module = line[5..import_index].trim();
            if !module.is_empty() && module != "." && module != ".." {
                dependencies.push(module.to_string());
            }
        }
    }
    
    fn normalize_path(&self, path: &Path) -> String {
        path.to_string_lossy().to_string()
    }
    
    fn resolve_dependency(&self, source_path: &str, dependency: &str) -> String {
        if dependency.starts_with("std") || dependency.contains("std::") || 
           dependency.starts_with("crate") || dependency.contains("crate::") {
            return dependency.to_string();
        }
        
        let source_dir = Path::new(source_path).parent().unwrap_or(Path::new(""));
        let dependency_path = source_dir.join(dependency);
        
        dependency_path.to_string_lossy().to_string()
    }
}
