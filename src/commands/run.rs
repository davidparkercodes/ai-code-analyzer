use crate::analyzer::Analyzer;
use crate::output::style;
use crate::util::parallel::{log_parallel_status, parse_parallel_flag, ParallelProcessing};

pub fn execute(path: String, no_parallel: bool) -> i32 {
    let parallel_enabled = parse_parallel_flag(no_parallel);
    
    let mut analyzer = initialize_analyzer(parallel_enabled);
    
    log_parallel_status(analyzer.is_parallel());
    
    let analysis_result = perform_codebase_analysis(&mut analyzer, &path);
    
    match analysis_result {
        Ok(_) => 0,
        Err(error_message) => {
            style::print_error(&error_message);
            1
        }
    }
}

fn initialize_analyzer(parallel_enabled: bool) -> Analyzer {
    Analyzer::new()
        .with_parallel(parallel_enabled)
}

fn perform_codebase_analysis(
    analyzer: &mut Analyzer, 
    directory_path: &str
) -> Result<(), String> {
    analyzer.analyze(directory_path)
        .map_err(|error| format!("Error analyzing directory: {}", error))
}