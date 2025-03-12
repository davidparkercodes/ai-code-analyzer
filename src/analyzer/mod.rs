use std::path::Path;
use crate::metrics::CodeMetrics;

pub struct Analyzer {
    metrics: CodeMetrics,
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer {
            metrics: CodeMetrics::new(),
        }
    }

    pub fn analyze<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        println!("Analyzing directory: {}", path.as_ref().display());
        
        // Collect code metrics
        self.metrics.analyze_directory(&path)?;
        
        // Print metrics summary
        self.metrics.print_summary();
        
        Ok(())
    }
}