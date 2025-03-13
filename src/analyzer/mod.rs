pub mod file_analyzer;

use crate::cache::AnalysisCache;
use crate::dependency::dependency_analyzer::DependencyAnalyzer;
use crate::dependency::dependency_reporter::DependencyReporter;
use crate::metrics::collector::MetricsCollector;
use crate::metrics::reporter::MetricsReporter;
use crate::output::style::*;
use std::path::Path;
use std::sync::Arc;

pub struct Analyzer {
    collector: MetricsCollector,
    reporter: MetricsReporter,
    dependency_analyzer: DependencyAnalyzer,
    dependency_reporter: DependencyReporter,
    cache: Arc<AnalysisCache>,
    parallel: bool,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    pub fn new() -> Self {
        let cache = Arc::new(AnalysisCache::new());
        let collector = MetricsCollector::new().with_parallel(true);
        let dependency_analyzer = DependencyAnalyzer::with_cache(Arc::clone(&cache));
        
        Analyzer {
            collector,
            reporter: MetricsReporter::new(),
            dependency_analyzer,
            dependency_reporter: DependencyReporter::new(),
            cache,
            parallel: true,
        }
    }
    
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    pub fn analyze<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        print_info(&format!("Analyzing directory: {}", path.as_ref().display()));
        print_info(&format!("Parallel processing: {}", if self.parallel { "enabled" } else { "disabled" }));
        print_info("Caching: enabled");
        
        let start_time = std::time::Instant::now();
        
        let metrics = self.collector.collect_metrics(&path)?;
        let metrics_time = start_time.elapsed();
        
        self.reporter.report(&metrics);
        print_success(&format!("Metrics analysis completed in {:.2?}", metrics_time));
        
        println!("");
        
        let dep_start_time = std::time::Instant::now();
        match self.dependency_analyzer.analyze_dependencies(&path) {
            Ok(graph) => {
                let dep_time = dep_start_time.elapsed();
                self.dependency_reporter.report(&graph);
                print_success(&format!("Dependency analysis completed in {:.2?}", dep_time));
            }
            Err(e) => {
                print_warning(&format!("Dependency analysis error: {}", e));
                print_warning("Continuing with partial results...");
            }
        }
        
        let total_time = start_time.elapsed();
        print_success(&format!("Total analysis time: {:.2?}", total_time));

        Ok(())
    }
}
