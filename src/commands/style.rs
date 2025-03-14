use crate::style_analyzer::{StyleAnalyzer, StyleReport};
use crate::output::style;
use crate::util::error::{AppError, AppResult, handle_command_error};
use crate::util::parallel::{log_parallel_status, parse_parallel_flag};
use std::time::Instant;
use std::path::Path;

pub fn execute(path: String, output: Option<String>, no_parallel: bool) -> i32 {
    match execute_style_command(path, output, no_parallel) {
        Ok(_) => 0,
        Err(error) => handle_command_error(&error)
    }
}

fn execute_style_command(
    path: String, 
    output: Option<String>, 
    no_parallel: bool
) -> AppResult<()> {
    let parallel_enabled = parse_parallel_flag(no_parallel);
    let analyzer = StyleAnalyzer::new();
    
    display_analysis_header(&path);
    log_parallel_status(parallel_enabled);
    
    let start_time = Instant::now();
    let report = analyze_code_style(&analyzer, &path)?;
    
    display_style_report(&report, start_time);
    
    if let Some(output_path) = output {
        export_style_guide(&report, output_path)?;
    }
    
    Ok(())
}

fn display_analysis_header(directory_path: &str) {
    style::print_header("Analyzing Code Style");
    style::print_info(&format!("Analyzing directory: {}", directory_path));
}

fn analyze_code_style(
    analyzer: &StyleAnalyzer, 
    directory_path: &str
) -> AppResult<StyleReport> {
    analyzer.analyze_codebase(directory_path)
        .map_err(|error| AppError::StyleAnalysis(format!("Error analyzing code style: {}", error)))
}

fn display_style_report(report: &StyleReport, start_time: Instant) {
    let elapsed = start_time.elapsed();
    println!("{}", report);
    style::print_success(&format!("Style analysis completed in {:.2?}", elapsed));
}

fn export_style_guide(report: &StyleReport, output_path: String) -> AppResult<()> {
    if let Some(style_guide) = report.get_style_guide() {
        write_style_guide_to_file(&style_guide, &output_path)?;
    }
    Ok(())
}

fn write_style_guide_to_file(content: &str, file_path: &str) -> AppResult<()> {
    let path = if file_path.starts_with('/') {
        Path::new(file_path).to_path_buf()
    } else {
        crate::output::path::create_output_path("style", file_path, "md")?
    };
    
    std::fs::write(&path, content)
        .map_err(|error| AppError::FileSystem { 
            path: path.clone(), 
            message: format!("Error writing style guide: {}", error) 
        })?;
    
    style::print_success(&format!("Style guide exported to {}", path.display()));
    Ok(())
}
