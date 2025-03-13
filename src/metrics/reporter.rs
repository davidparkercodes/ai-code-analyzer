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
            StyledText::new("--------------------").foreground(Color::Cyan)
        );
        println!(
            "{} {}",
            highlight("Total Directories:"),
            metrics.total_directories
        );
        println!("{} {}", highlight("Total Files:"), metrics.total_files);
        println!(
            "{} {}",
            highlight("Total Lines of Code:"),
            metrics.lines_of_code
        );
        println!(
            "{} {}",
            highlight("Total Blank Lines:"),
            metrics.blank_lines
        );
        println!(
            "{} {}",
            highlight("Total Comment Lines:"),
            metrics.comment_lines
        );

        if !metrics.by_language.is_empty() {
            println!();
            print_header("Breakdown by Language:");
            println!(
                "{}",
                StyledText::new("---------------------").foreground(Color::Cyan)
            );

            let mut languages: Vec<(&String, &crate::metrics::models::LanguageMetrics)> =
                metrics.by_language.iter().collect();

            languages.sort_by(|a, b| b.1.lines_of_code.cmp(&a.1.lines_of_code));

            for (language, lang_metrics) in languages {
                println!(
                    "{}: {} files, {} lines of code",
                    StyledText::new(language)
                        .foreground(Color::Green)
                        .style(Style::Bold),
                    lang_metrics.files,
                    lang_metrics.lines_of_code
                );
            }
        }
    }
}
