/// A trait for types that support configuring parallel processing
pub trait ParallelProcessing {
    /// Configure parallel processing mode
    fn with_parallel(self, parallel: bool) -> Self;
    
    /// Get the current parallel processing setting
    fn is_parallel(&self) -> bool;
}

/// Helper function to parse common command-line args for parallel processing
pub fn parse_parallel_flag(no_parallel: bool) -> bool {
    !no_parallel
}

/// Helper function to log parallel processing status
pub fn log_parallel_status(is_parallel: bool) {
    if is_parallel {
        crate::output::style::print_info("⚡ Parallel processing: enabled");
    } else {
        crate::output::style::print_info("🔄 Sequential processing: enabled");
    }
}