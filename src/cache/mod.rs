use dashmap::DashMap;
use std::hash::Hash;
use std::path::Path;
use std::time::SystemTime;

pub struct AnalysisCache {
    file_content_cache: DashMap<String, (String, SystemTime)>,
    language_cache: DashMap<String, (String, SystemTime)>,
    metrics_cache: DashMap<String, (crate::metrics::models::FileMetrics, SystemTime)>,
    dependency_cache: DashMap<String, (Vec<String>, SystemTime)>,
}

impl Default for AnalysisCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisCache {
    pub fn new() -> Self {
        Self {
            file_content_cache: DashMap::new(),
            language_cache: DashMap::new(),
            metrics_cache: DashMap::new(),
            dependency_cache: DashMap::new(),
        }
    }

    pub fn get_file_content(&self, path: &str) -> Option<String> {
        self.get_if_not_modified(path, &self.file_content_cache)
            .map(|(content, _)| content)
    }

    pub fn cache_file_content(&self, path: &str, content: String) {
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                self.file_content_cache.insert(path.to_string(), (content, modified));
            }
        }
    }

    pub fn get_language(&self, path: &str) -> Option<String> {
        self.get_if_not_modified(path, &self.language_cache)
            .map(|(lang, _)| lang)
    }

    pub fn cache_language(&self, path: &str, language: String) {
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                self.language_cache.insert(path.to_string(), (language, modified));
            }
        }
    }

    pub fn get_metrics(&self, path: &str) -> Option<crate::metrics::models::FileMetrics> {
        self.get_if_not_modified(path, &self.metrics_cache)
            .map(|(metrics, _)| metrics)
    }

    pub fn cache_metrics(&self, path: &str, metrics: crate::metrics::models::FileMetrics) {
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                self.metrics_cache.insert(path.to_string(), (metrics, modified));
            }
        }
    }

    pub fn get_dependencies(&self, path: &str) -> Option<Vec<String>> {
        self.get_if_not_modified(path, &self.dependency_cache)
            .map(|(deps, _)| deps)
    }

    pub fn cache_dependencies(&self, path: &str, dependencies: Vec<String>) {
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                self.dependency_cache.insert(path.to_string(), (dependencies, modified));
            }
        }
    }

    fn get_if_not_modified<K, V>(&self, key: &str, cache: &DashMap<K, (V, SystemTime)>) -> Option<(V, SystemTime)>
    where
        K: Eq + Hash + From<String> + Clone,
        V: Clone,
    {
        let k = K::from(key.to_string());
        if let Some(entry) = cache.get(&k) {
            let (value, cached_time) = entry.value();
            if let Ok(metadata) = std::fs::metadata(key) {
                if let Ok(current_time) = metadata.modified() {
                    if current_time <= *cached_time {
                        return Some((value.clone(), *cached_time));
                    }
                }
            }
        }
        None
    }

    pub fn clear(&self) {
        self.file_content_cache.clear();
        self.language_cache.clear();
        self.metrics_cache.clear();
        self.dependency_cache.clear();
    }

    pub fn purge_stale_entries(&self) {
        self.purge_stale_entries_from_cache(&self.file_content_cache);
        self.purge_stale_entries_from_cache(&self.language_cache);
        self.purge_stale_entries_from_cache(&self.metrics_cache);
        self.purge_stale_entries_from_cache(&self.dependency_cache);
    }

    fn purge_stale_entries_from_cache<K, V>(&self, cache: &DashMap<K, (V, SystemTime)>)
    where
        K: Eq + Hash + From<String> + AsRef<str> + Clone,
    {
        let keys_to_remove: Vec<_> = cache
            .iter()
            .filter_map(|entry| {
                let key = entry.key();
                let key_str = key.as_ref().to_string();
                if !Path::new(&key_str).exists() {
                    Some(key.clone())
                } else if let Ok(metadata) = std::fs::metadata(&key_str) {
                    if let Ok(current_time) = metadata.modified() {
                        let (_, cached_time) = entry.value();
                        if current_time > *cached_time {
                            return Some(key.clone());
                        }
                    }
                    None
                } else {
                    Some(key.clone())
                }
            })
            .collect();

        for key in keys_to_remove {
            cache.remove(&key);
        }
    }
}