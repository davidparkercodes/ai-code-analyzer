use crate::style_analyzer::{StyleAnalyzer, StyleReport};
use crate::output::style;
use crate::util::parallel::{log_parallel_status, parse_parallel_flag};
use std::time::Instant;
use std::path::Path;

pub fn execute(path: String, output: Option<String>, no_parallel: bool) -> i32 {
    let parallel_enabled = parse_parallel_flag(no_parallel);
    let analyzer = StyleAnalyzer::new();
    
    display_analysis_header(&path);
    log_parallel_status(parallel_enabled);
    
    let start_time = Instant::now();
    let analysis_result = analyze_code_style(&analyzer, &path);
    
    match analysis_result {
        Ok(report) => {
            display_style_report(&report, start_time);
            
            if let Some(output_path) = output {
                if let Err(exit_code) = export_style_guide(&report, output_path) {
                    return exit_code;
                }
            }
            0
        }
        Err(error_message) => {
            style::print_error(&error_message);
            1
        }
    }
}

fn display_analysis_header(directory_path: &str) {
    style::print_header("Analyzing Code Style");
    style::print_info(&format!("Analyzing directory: {}", directory_path));
}

fn analyze_code_style(
    analyzer: &StyleAnalyzer, 
    directory_path: &str
) -> Result<StyleReport, String> {
    analyzer.analyze_codebase(directory_path)
        .map_err(|error| format!("Error analyzing code style: {}", error))
}

fn display_style_report(report: &StyleReport, start_time: Instant) {
    let elapsed = start_time.elapsed();
    println!("{}", report);
    style::print_success(&format!("Style analysis completed in {:.2?}", elapsed));
}

fn export_style_guide(report: &StyleReport, output_path: String) -> Result<(), i32> {
    if let Some(style_guide) = report.get_style_guide() {
        write_style_guide_to_file(&style_guide, &output_path)
    } else {
        Ok(())
    }
}

fn write_style_guide_to_file(content: &str, file_path: &str) -> Result<(), i32> {
    let path = Path::new(file_path);
    match std::fs::write(path, content) {
        Ok(_) => {
            style::print_success(&format!("Style guide exported to {}", file_path));
            Ok(())
        }
        Err(error) => {
            style::print_error(&format!("Error writing style guide: {}", error));
            Err(1)
        }
    }
}