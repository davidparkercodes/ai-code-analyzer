pub mod file_analyzer;

use crate::dependency::dependency_analyzer::DependencyAnalyzer;
use crate::dependency::dependency_reporter::DependencyReporter;
use crate::metrics::collector::MetricsCollector;
use crate::metrics::reporter::MetricsReporter;
use crate::output::style::*;
use std::path::Path;

pub struct Analyzer {
    collector: MetricsCollector,
    reporter: MetricsReporter,
    dependency_analyzer: DependencyAnalyzer,
    dependency_reporter: DependencyReporter,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer {
            collector: MetricsCollector::new(),
            reporter: MetricsReporter::new(),
            dependency_analyzer: DependencyAnalyzer::new(),
            dependency_reporter: DependencyReporter::new(),
        }
    }

    pub fn analyze<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        print_info(&format!("Analyzing directory: {}", path.as_ref().display()));

        let metrics = self.collector.collect_metrics(&path)?;
        self.reporter.report(&metrics);
        
        println!("");
        
        match self.dependency_analyzer.analyze_dependencies(&path) {
            Ok(graph) => {
                self.dependency_reporter.report(&graph);
            }
            Err(e) => {
                print_warning(&format!("Dependency analysis error: {}", e));
                print_warning("Continuing with partial results...");
            }
        }

        Ok(())
    }
}
