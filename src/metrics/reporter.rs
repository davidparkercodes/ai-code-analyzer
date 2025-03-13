use crate::metrics::models::CodeMetrics;
use crate::output::style::*;

pub struct MetricsReporter;

impl Default for MetricsReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsReporter {
    pub fn new() -> Self {
        MetricsReporter
    }

    pub fn report(&self, metrics: &CodeMetrics) {
        println!();
        print_header("Code Metrics Summary:");
        println!(
            "{}",
            StyledText::new("====================").foreground(ThemeColors::SEPARATOR)
        );
        
        // Print overall metrics
        self.print_overall_metrics(metrics);
        
        // Print production code metrics
        println!();
        print_header("Production Code Metrics:");
        println!(
            "{}",
            StyledText::new("========================").foreground(ThemeColors::SEPARATOR)
        );
        self.print_production_metrics(metrics);
        
        // Print test code metrics
        println!();
        print_header("Test Code Metrics:");
        println!(
            "{}",
            StyledText::new("=================").foreground(ThemeColors::SEPARATOR)
        );
        self.print_test_metrics(metrics);
    }
    
    fn print_overall_metrics(&self, metrics: &CodeMetrics) {
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
                    .foreground(ThemeColors::NUMBER)
                    .style(Style::Bold)
            );
        }
    }
    
    fn print_production_metrics(&self, metrics: &CodeMetrics) {
        // Production metrics
        let labels = [
            "Files:",
            "Lines of Code:",
            "Blank Lines:",
            "Comment Lines:",
        ];
        
        let values = [
            metrics.prod_files,
            metrics.prod_lines_of_code,
            metrics.prod_blank_lines,
            metrics.prod_comment_lines,
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
                    .foreground(ThemeColors::NUMBER)
                    .style(Style::Bold)
            );
        }
    }
    
    fn print_test_metrics(&self, metrics: &CodeMetrics) {
        // Test metrics
        let labels = [
            "Files:",
            "Lines of Code:",
            "Blank Lines:",
            "Comment Lines:",
        ];
        
        let values = [
            metrics.test_files,
            metrics.test_lines_of_code,
            metrics.test_blank_lines,
            metrics.test_comment_lines,
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
                    .foreground(ThemeColors::NUMBER)
                    .style(Style::Bold)
            );
        }

        // Show overall breakdown by language
        if !metrics.by_language.is_empty() {
            println!();
            print_header("Overall Breakdown by Language:");
            println!(
                "{}",
                StyledText::new("===========================").foreground(ThemeColors::SEPARATOR)
            );

            self.print_language_breakdown(&metrics.by_language);
        }
        
        // Show production code breakdown by language
        if !metrics.prod_by_language.is_empty() {
            println!();
            print_header("Production Code by Language:");
            println!(
                "{}",
                StyledText::new("===========================").foreground(ThemeColors::SEPARATOR)
            );

            self.print_language_breakdown(&metrics.prod_by_language);
        }
        
        // Show test code breakdown by language
        if !metrics.test_by_language.is_empty() {
            println!();
            print_header("Test Code by Language:");
            println!(
                "{}",
                StyledText::new("=====================").foreground(ThemeColors::SEPARATOR)
            );

            self.print_language_breakdown(&metrics.test_by_language);
        }
    }
    
    fn print_language_breakdown(&self, language_map: &std::collections::HashMap<String, crate::metrics::models::LanguageMetrics>) {
        let mut languages: Vec<(&String, &crate::metrics::models::LanguageMetrics)> =
            language_map.iter().collect();

        languages.sort_by(|a, b| b.1.lines_of_code.cmp(&a.1.lines_of_code));
        
        // Skip if empty
        if languages.is_empty() {
            println!("No language data available.");
            return;
        }
        
        // Constants for table formatting
        const COL_SPACING: usize = 6; // Spacing between columns
        let language_header = "Language";
        let files_header = "Files";
        let loc_header = "Lines of Code";
        
        // Calculate column widths
        let max_lang_len = languages.iter()
            .map(|(lang, _)| lang.len())
            .max()
            .unwrap_or(0)
            .max(language_header.len());
            
        let max_files = languages.iter()
            .map(|(_, metrics)| metrics.files.to_string().len())
            .max()
            .unwrap_or(0)
            .max(files_header.len());
            
        let max_loc = languages.iter()
            .map(|(_, metrics)| metrics.lines_of_code.to_string().len())
            .max()
            .unwrap_or(0)
            .max(loc_header.len());
        
        // Calculate column widths
        let lang_width = max_lang_len + COL_SPACING;
        let files_width = max_files + COL_SPACING;
        let loc_width = max_loc + COL_SPACING;
        
        // Print header
        let header = format!("{:<lang_width$}{:>files_width$}{:>loc_width$}",
            language_header,
            files_header,
            loc_header,
            lang_width = lang_width,
            files_width = files_width,
            loc_width = loc_width
        );
        println!("{}", StyledText::new(&header).foreground(ThemeColors::TABLE_HEADER));
        
        // Print header separator
        let lang_separator = "-".repeat(language_header.len());
        let files_separator = "-".repeat(files_header.len());
        let loc_separator = "-".repeat(loc_header.len());
        
        let separator = format!("{:<lang_width$}{:>files_width$}{:>loc_width$}",
            lang_separator,
            files_separator,
            loc_separator,
            lang_width = lang_width,
            files_width = files_width,
            loc_width = loc_width
        );
        println!("{}", StyledText::new(&separator).foreground(ThemeColors::TABLE_HEADER));

        // Print language rows
        for (language, lang_metrics) in languages {
            // Use a different approach: print each column separately with correct spacing
            print!("{}", StyledText::new(language)
                .foreground(ThemeColors::LANGUAGE)
                .style(Style::Bold));
            
            // Calculate required padding between columns
            let lang_padding = lang_width - language.len();
            print!("{}", " ".repeat(lang_padding));
            
            let files_str = lang_metrics.files.to_string();
            let files_padding = files_width - files_str.len();
            print!("{}{}", " ".repeat(files_padding), 
                StyledText::new(&files_str).foreground(ThemeColors::NUMBER));
            
            let loc_str = lang_metrics.lines_of_code.to_string();
            let loc_padding = loc_width - loc_str.len();
            println!("{}{}", " ".repeat(loc_padding),
                StyledText::new(&loc_str).foreground(ThemeColors::NUMBER));
        }
    }
}
