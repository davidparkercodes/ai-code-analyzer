pub mod ai;
pub mod analyzer;
pub mod cache;
pub mod commands;
pub mod dependency;
pub mod description;
pub mod metrics;
pub mod output;
pub mod style_analyzer;
pub mod util;

// Re-export the modules for testing
pub use crate::ai::*;
pub use crate::analyzer::*;
pub use crate::cache::*;
pub use crate::commands::*;
pub use crate::dependency::*;
pub use crate::description::*;
pub use crate::metrics::*;
pub use crate::output::*;
pub use crate::style_analyzer::*;
pub use crate::util::*;
