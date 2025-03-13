pub mod analyzer;
pub mod dependency;
pub mod metrics;
pub mod output;
pub mod style;

// Re-export specific modules for testing
pub use crate::analyzer::*;
pub use crate::dependency::*;

// Re-export metrics module - but avoid re-exporting the metrics::models
pub use crate::metrics::collector;
pub use crate::metrics::language;
pub use crate::metrics::reporter as metrics_reporter;

// Re-export style module - but avoid re-exporting the style::models
// which would conflict with the metrics::models
pub use crate::style::detector;
pub use crate::style::reporter as style_reporter;
