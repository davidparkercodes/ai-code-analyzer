use crate::analyzer::Analyzer;
use crate::output::style;
use crate::util::parallel::{log_parallel_status, parse_parallel_flag, ParallelProcessing};

pub fn execute(path: String, no_parallel: bool) {
    let is_parallel = parse_parallel_flag(no_parallel);
    
    let mut analyzer = Analyzer::new()
        .with_parallel(is_parallel);
    
    // Use is_parallel method from the trait
    log_parallel_status(analyzer.is_parallel());
        
    if let Err(e) = analyzer.analyze(&path) {
        style::print_error(&format!("Error analyzing directory: {}", e));
        std::process::exit(1);
    }
}