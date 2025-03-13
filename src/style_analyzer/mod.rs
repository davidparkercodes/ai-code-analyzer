mod detector;
pub mod pattern;
mod report;

use std::path::Path;

pub use detector::StyleDetector;
pub use report::StyleReport;

// Re-export the needed types from pattern module
// These are needed for tests and public API
pub use pattern::{StylePattern, StyleRule};

pub struct StyleAnalyzer {
    detector: StyleDetector,
}

impl Default for StyleAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleAnalyzer {
    pub fn new() -> Self {
        StyleAnalyzer {
            detector: StyleDetector::new(),
        }
    }

    pub fn analyze_codebase<P: AsRef<Path>>(&self, dir_path: P) -> Result<StyleReport, String> {
        self.detector.detect_patterns(dir_path)
    }
}