pub mod analyzer;
pub mod dependency;
pub mod metrics;
pub mod output;

// Re-export the modules for testing
pub use crate::analyzer::*;
pub use crate::dependency::*;
pub use crate::metrics::*;
pub use crate::output::*;
