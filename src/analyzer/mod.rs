pub mod file_analyzer;

use crate::metrics::collector::MetricsCollector;
use crate::metrics::reporter::MetricsReporter;
use crate::output::style::*;
use std::path::Path;

pub struct Analyzer {
    collector: MetricsCollector,
    reporter: MetricsReporter,
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
        }
    }

    pub fn analyze<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        print_info(&format!("Analyzing directory: {}", path.as_ref().display()));

        let metrics = self.collector.collect_metrics(&path)?;
        self.reporter.report(&metrics);

        Ok(())
    }
}
