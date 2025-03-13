use crate::style_analyzer::StyleAnalyzer;
use crate::output::style;
use crate::util::parallel::{log_parallel_status, parse_parallel_flag};
use std::time::Instant;

pub fn execute(path: String, output: Option<String>, no_parallel: bool) {
    let is_parallel = parse_parallel_flag(no_parallel);
    let analyzer = StyleAnalyzer::new();
    
    style::print_header("Analyzing Code Style");
    style::print_info(&format!("Analyzing directory: {}", path));
    
    log_parallel_status(is_parallel);
    
    let start_time = Instant::now();
    match analyzer.analyze_codebase(&path) {
        Ok(report) => {
            let elapsed = start_time.elapsed();
            
            // Print the report to console
            println!("{}", report);
            style::print_success(&format!("Style analysis completed in {:.2?}", elapsed));
            
            // Export the style guide if requested
            if let Some(output_path) = output {
                if let Some(style_guide) = report.get_style_guide() {
                    let path_display = output_path.clone();
                    match std::fs::write(output_path, style_guide) {
                        Ok(_) => {
                            style::print_success(&format!("Style guide exported to {}", path_display));
                        }
                        Err(e) => {
                            style::print_error(&format!("Error writing style guide: {}", e));
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Err(e) => {
            style::print_error(&format!("Error analyzing code style: {}", e));
            std::process::exit(1);
        }
    }
}