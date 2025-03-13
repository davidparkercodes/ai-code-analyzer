use crate::metrics::models::CodeMetrics;
use crate::output::style::*;

pub struct MetricsReporter;

impl MetricsReporter {
    pub fn new() -> Self {
        MetricsReporter
    }

    pub fn report(&self, metrics: &CodeMetrics) {
        println!();
        print_header("Code Metrics Summary:");
        println!(
            "{}",
            StyledText::new("====================").foreground(Color::Cyan)
        );
        
        // Format metrics as a table with aligned numbers
        let labels = [
            "Total Directories:",
            "Total Files:",
            "Total Lines of Code:",
            "Total Blank Lines:",
            "Total Comment Lines:",
        ];
        
        let values = [
            metrics.total_directories,
            metrics.total_files,
            metrics.lines_of_code,
            metrics.blank_lines,
            metrics.comment_lines,
        ];
        
        // Find the longest label for alignment
        let max_label_len = labels.iter().map(|l| l.len()).max().unwrap_or(0);
        
        // Find the longest value for alignment
        let max_value_len = values.iter().map(|v| v.to_string().len()).max().unwrap_or(0);
        
        // Print summary table
        for (label, value) in labels.iter().zip(values.iter()) {
            println!(
                "{}{}    {}",
                highlight(label),
                " ".repeat(max_label_len - label.len()),
                StyledText::new(&format!("{:>width$}", value, width = max_value_len))
                    .foreground(Color::Cyan)
                    .style(Style::Bold)
            );
        }

        if !metrics.by_language.is_empty() {
            println!();
            print_header("Breakdown by Language:");
            println!(
                "{}",
                StyledText::new("=====================").foreground(Color::Cyan)
            );

            let mut languages: Vec<(&String, &crate::metrics::models::LanguageMetrics)> =
                metrics.by_language.iter().collect();

            languages.sort_by(|a, b| b.1.lines_of_code.cmp(&a.1.lines_of_code));
            
            // Find the longest language name and maximum values for alignment
            let max_lang_len = languages.iter().map(|(lang, _)| lang.len()).max().unwrap_or(0);
            let max_files = languages.iter()
                .map(|(_, metrics)| metrics.files.to_string().len())
                .max().unwrap_or(0);
            let max_loc = languages.iter()
                .map(|(_, metrics)| metrics.lines_of_code.to_string().len())
                .max().unwrap_or(0);
            
            // Print header
            println!("{}", 
                StyledText::new(&format!(
                    "{:<width_lang$}    {:>width_files$}    {:>width_loc$}",
                    "Language",
                    "Files",
                    "Lines of Code",
                    width_lang = max_lang_len,
                    width_files = max_files.max(5), // min width of 5 for "Files"
                    width_loc = max_loc.max(13),    // min width of 13 for "Lines of Code"
                )).foreground(Color::Blue)
            );
            
            println!("{}", 
                StyledText::new(&format!(
                    "{:<width_lang$}    {:>width_files$}    {:>width_loc$}",
                    "--------",
                    "-----",
                    "-------------",
                    width_lang = max_lang_len,
                    width_files = max_files.max(5),
                    width_loc = max_loc.max(13),
                )).foreground(Color::Blue)
            );

            // Print language rows
            for (language, lang_metrics) in languages {
                println!(
                    "{:<width$}    {:>width_files$}    {:>width_loc$}",
                    StyledText::new(language)
                        .foreground(Color::Green)
                        .style(Style::Bold),
                    StyledText::new(&lang_metrics.files.to_string()).foreground(Color::White),
                    StyledText::new(&lang_metrics.lines_of_code.to_string()).foreground(Color::White),
                    width = max_lang_len,
                    width_files = max_files.max(5),
                    width_loc = max_loc.max(13),
                );
            }
        }
    }
}
