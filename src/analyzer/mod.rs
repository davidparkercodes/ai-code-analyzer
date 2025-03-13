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
    parallel: bool,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

use crate::util::parallel::ParallelProcessing;

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
            parallel: true,
        }
    }
}

impl ParallelProcessing for Analyzer {
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

impl Analyzer {
    pub fn analyze<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        self.print_analysis_header(&path);
        
        let start_time = std::time::Instant::now();
        
        self.perform_metrics_analysis(&path, &start_time)?;
        self.perform_dependency_analysis(&path);
        
        self.print_total_analysis_time(start_time);
        Ok(())
    }
    
    fn print_analysis_header<P: AsRef<Path>>(&self, path: P) {
        print_info(&format!("Analyzing directory: {}", path.as_ref().display()));
        print_info(&format!("Parallel processing: {}", if self.parallel { "enabled" } else { "disabled" }));
        print_info("Caching: enabled");
    }
    
    fn perform_metrics_analysis<P: AsRef<Path>>(&mut self, path: P, start_time: &std::time::Instant) -> Result<(), String> {
        let metrics = self.collector.collect_metrics(&path)?;
        let metrics_time = start_time.elapsed();
        
        self.reporter.report(&metrics);
        print_success(&format!("Metrics analysis completed in {:.2?}", metrics_time));
        
        println!("");
        Ok(())
    }
    
    fn perform_dependency_analysis<P: AsRef<Path>>(&mut self, path: P) {
        let dep_start_time = std::time::Instant::now();
        
        match self.dependency_analyzer.analyze_dependencies(&path) {
            Ok(graph) => {
                let dep_time = dep_start_time.elapsed();
                self.dependency_reporter.report(&graph);
                print_success(&format!("Dependency analysis completed in {:.2?}", dep_time));
            }
            Err(error) => {
                print_warning(&format!("Dependency analysis error: {}", error));
                print_warning("Continuing with partial results...");
            }
        }
    }
    
    fn print_total_analysis_time(&self, start_time: std::time::Instant) {
        let total_time = start_time.elapsed();
        print_success(&format!("Total analysis time: {:.2?}", total_time));
    }
}
