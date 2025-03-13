use crate::metrics::models::CodeMetrics;

pub struct MetricsReporter;

impl MetricsReporter {
    pub fn new() -> Self {
        MetricsReporter
    }
    
    pub fn report(&self, metrics: &CodeMetrics) {
        println!("\nCode Metrics Summary:");
        println!("--------------------");
        println!("Total Directories: {}", metrics.total_directories);
        println!("Total Files: {}", metrics.total_files);
        println!("Total Lines of Code: {}", metrics.lines_of_code);
        println!("Total Blank Lines: {}", metrics.blank_lines);
        println!("Total Comment Lines: {}", metrics.comment_lines);
        
        if !metrics.by_language.is_empty() {
            println!("\nBreakdown by Language:");
            println!("---------------------");
            
            let mut languages: Vec<(&String, &crate::metrics::models::LanguageMetrics)> = 
                metrics.by_language.iter().collect();
            
            languages.sort_by(|a, b| b.1.lines_of_code.cmp(&a.1.lines_of_code));
            
            for (language, lang_metrics) in languages {
                println!("{}: {} files, {} lines of code", 
                    language, lang_metrics.files, lang_metrics.lines_of_code);
            }
        }
    }
}