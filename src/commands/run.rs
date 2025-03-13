use crate::analyzer::Analyzer;
use crate::output::style;
use crate::util::error::{AppError, AppResult, handle_command_error};
use crate::util::parallel::{log_parallel_status, parse_parallel_flag, ParallelProcessing};

pub fn execute(path: String, no_parallel: bool) -> i32 {
    match execute_run_command(path, no_parallel) {
        Ok(_) => 0,  // Success exit code
        Err(error) => handle_command_error(&error)
    }
}

fn execute_run_command(path: String, no_parallel: bool) -> AppResult<()> {
    let parallel_enabled = parse_parallel_flag(no_parallel);
    
    let mut analyzer = initialize_analyzer(parallel_enabled);
    
    log_parallel_status(analyzer.is_parallel());
    
    perform_codebase_analysis(&mut analyzer, &path)?;
    
    Ok(())
}

fn initialize_analyzer(parallel_enabled: bool) -> Analyzer {
    Analyzer::new()
        .with_parallel(parallel_enabled)
}

fn perform_codebase_analysis(
    analyzer: &mut Analyzer, 
    directory_path: &str
) -> AppResult<()> {
    analyzer.analyze(directory_path)
        .map_err(|error| AppError::Analysis(format!("Error analyzing directory: {}", error)))
}