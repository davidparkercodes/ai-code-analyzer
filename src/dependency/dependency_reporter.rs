use crate::dependency::dependency_graph::DependencyGraph;
use crate::output::style::*;
use std::fs;
use std::path::Path;

pub struct DependencyReporter;

impl Default for DependencyReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyReporter {
    pub fn new() -> Self {
        DependencyReporter
    }
    
    pub fn report(&self, graph: &DependencyGraph) {
        println!();
        print_header("Dependency Analysis:");
        println!(
            "{}",
            StyledText::new("====================").foreground(ThemeColors::SEPARATOR)
        );
        
        self.print_summary(graph);
        
        if !graph.get_nodes().is_empty() {
            self.print_top_dependencies(graph);
        }
        
        self.print_circular_dependencies(graph);
    }
    
    fn print_summary(&self, graph: &DependencyGraph) {
        println!();
        print_header("Dependency Summary:");
        println!(
            "{}",
            StyledText::new("====================").foreground(ThemeColors::SEPARATOR)
        );
        
        let node_count = graph.get_nodes().len();
        
        let mut summary_data = Vec::new();
        summary_data.push(("Total Files", node_count.to_string()));
        
        if node_count > 0 {
            let mut total_dependencies = 0;
            let mut max_dependencies = 0;
            let mut max_dependents = 0;
            let mut most_depended_node = String::new();
            let mut most_dependent_node = String::new();
            
            for node in graph.get_nodes() {
                let dependencies = graph.get_dependencies(node);
                let dependents = graph.get_dependents(node);
                
                total_dependencies += dependencies.len();
                
                if dependencies.len() > max_dependencies {
                    max_dependencies = dependencies.len();
                    most_dependent_node = node.clone();
                }
                
                if dependents.len() > max_dependents {
                    max_dependents = dependents.len();
                    most_depended_node = node.clone();
                }
            }
            
            summary_data.push(("Average Dependencies", format!("{:.2}", total_dependencies as f64 / node_count as f64)));
            
            if !most_dependent_node.is_empty() {
                summary_data.push(("Most Dependent File", most_dependent_node));
                summary_data.push(("Dependencies Count", max_dependencies.to_string()));
            }
            
            if !most_depended_node.is_empty() {
                summary_data.push(("Most Depended-on File", most_depended_node));
                summary_data.push(("Dependents Count", max_dependents.to_string()));
            }
        }
        
        let max_label_len = summary_data.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
        
        for (label, value) in summary_data {
            println!(
                "{}{}    {}",
                highlight(label),
                " ".repeat(max_label_len - label.len()),
                StyledText::new(&value)
                    .foreground(ThemeColors::NUMBER)
                    .style(Style::Bold)
            );
        }
    }
    
    fn print_circular_dependencies(&self, graph: &DependencyGraph) {
        let cycles = graph.find_circular_dependencies();
        
        println!();
        print_header("Circular Dependencies:");
        println!(
            "{}",
            StyledText::new("=====================").foreground(ThemeColors::SEPARATOR)
        );
        
        if cycles.is_empty() {
            println!("{}", StyledText::new("No circular dependencies found.").foreground(ThemeColors::LANGUAGE));
        } else {
            println!(
                "{}",
                StyledText::new(&format!("Found {} circular dependencies:", cycles.len()))
                    .foreground(Color::Yellow)
                    .style(Style::Bold)
            );
            println!();
            
            for (i, cycle) in cycles.iter().enumerate() {
                println!(
                    "{}",
                    StyledText::new(&format!("Cycle {}:", i + 1))
                        .foreground(Color::Yellow)
                        .style(Style::Bold)
                );
                
                for (j, node) in cycle.iter().enumerate() {
                    if j < cycle.len() - 1 {
                        print!("  ");
                        print!("{}", StyledText::new(node).foreground(ThemeColors::LABEL));
                        println!(" → ");
                    } else {
                        print!("  ");
                        print!("{}", StyledText::new(node).foreground(ThemeColors::LABEL));
                        println!(" → (back to start)");
                    }
                }
                println!();
            }
        }
    }
    
    fn print_top_dependencies(&self, graph: &DependencyGraph) {
        println!();
        print_header("Top Dependencies:");
        println!(
            "{}",
            StyledText::new("================").foreground(ThemeColors::SEPARATOR)
        );
        
        let mut nodes_with_counts: Vec<(String, usize, usize)> = graph
            .get_nodes()
            .iter()
            .map(|node| {
                let dependencies = graph.get_dependencies(node);
                let dependents = graph.get_dependents(node);
                (node.clone(), dependencies.len(), dependents.len())
            })
            .collect();
        
        nodes_with_counts.sort_by(|a, b| (b.1 + b.2).cmp(&(a.1 + a.2)));
        
        let top_nodes = if nodes_with_counts.len() > 10 {
            &nodes_with_counts[0..10]
        } else {
            &nodes_with_counts[..]
        };
        
        if top_nodes.is_empty() || (top_nodes[0].1 == 0 && top_nodes[0].2 == 0) {
            println!("{}", StyledText::new("No significant dependencies found.").foreground(ThemeColors::LANGUAGE));
            return;
        }
        
        const COL_SPACING: usize = 4;
        let file_header = "File";
        let deps_header = "Dependencies";
        let dependents_header = "Dependents";
        
        let max_filename_len = top_nodes
            .iter()
            .map(|(name, _, _)| {
                match name.split('/').last() {
                    Some(last) => last.len(),
                    None => name.len(),
                }
            })
            .max()
            .unwrap_or(0)
            .max(file_header.len());
            
        let max_deps = top_nodes
            .iter()
            .map(|(_, deps, _)| deps.to_string().len())
            .max()
            .unwrap_or(0)
            .max(deps_header.len());
            
        let max_dependents = top_nodes
            .iter()
            .map(|(_, _, dependents)| dependents.to_string().len())
            .max()
            .unwrap_or(0)
            .max(dependents_header.len());
        
        let filename_width = max_filename_len + COL_SPACING;
        let deps_width = max_deps + COL_SPACING;
        let dependents_width = max_dependents + COL_SPACING;
        
        let header = format!("{:<filename_width$}{:>deps_width$}{:>dependents_width$}",
            file_header,
            deps_header,
            dependents_header,
            filename_width = filename_width,
            deps_width = deps_width,
            dependents_width = dependents_width
        );
        println!("{}", StyledText::new(&header).foreground(ThemeColors::TABLE_HEADER));
        
        let file_separator = "-".repeat(file_header.len());
        let deps_separator = "-".repeat(deps_header.len());
        let dependents_separator = "-".repeat(dependents_header.len());
        
        let separator = format!("{:<filename_width$}{:>deps_width$}{:>dependents_width$}",
            file_separator,
            deps_separator,
            dependents_separator,
            filename_width = filename_width,
            deps_width = deps_width,
            dependents_width = dependents_width
        );
        println!("{}", StyledText::new(&separator).foreground(ThemeColors::TABLE_HEADER));
        
        for (name, deps, dependents) in top_nodes {
            let display_name = match name.split('/').last() {
                Some(last) => last,
                None => name,
            };
            
            print!("{}", StyledText::new(display_name)
                .foreground(ThemeColors::LANGUAGE)
                .style(Style::Bold));
            
            let filename_padding = filename_width - display_name.len();
            print!("{}", " ".repeat(filename_padding));
            
            let deps_str = deps.to_string();
            let deps_padding = deps_width - deps_str.len();
            print!("{}{}", " ".repeat(deps_padding), 
                StyledText::new(&deps_str).foreground(ThemeColors::NUMBER));
            
            let dependents_str = dependents.to_string();
            let dependents_padding = dependents_width - dependents_str.len();
            println!("{}{}", " ".repeat(dependents_padding),
                StyledText::new(&dependents_str).foreground(ThemeColors::NUMBER));
        }
    }
    
    pub fn export_dot<P: AsRef<Path>>(&self, graph: &DependencyGraph, output_path: P) -> Result<(), String> {
        let dot_content = graph.to_dot_format();
        let path_ref = output_path.as_ref();
        let path_str = path_ref.to_str().unwrap_or("");
        
        let final_path = if path_str.starts_with('/') {
            path_ref.to_path_buf()
        } else {
            match crate::output::path::create_output_path("dependencies", path_str, "dot") {
                Ok(p) => p,
                Err(e) => return Err(format!("Error creating output path: {}", e)),
            }
        };
        
        fs::write(&final_path, dot_content)
            .map_err(|e| format!("Failed to write DOT file: {}", e))?;
            
        println!("Dependency graph exported to {}", final_path.display());
        Ok(())
    }
}
