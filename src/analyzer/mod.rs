pub mod file_analyzer;

use std::path::Path;
use crate::metrics::collector::MetricsCollector;
use crate::metrics::reporter::MetricsReporter;

pub struct Analyzer {
    collector: MetricsCollector,
    reporter: MetricsReporter,
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer {
            collector: MetricsCollector::new(),
            reporter: MetricsReporter::new(),
        }
    }

    pub fn analyze<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        println!("Analyzing directory: {}", path.as_ref().display());
        
        let metrics = self.collector.collect_metrics(&path)?;
        self.reporter.report(&metrics);
        
        Ok(())
    }
}