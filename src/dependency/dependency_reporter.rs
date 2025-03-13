use crate::dependency::dependency_graph::DependencyGraph;
use crate::output::style;
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
        style::print_heading("Dependency Analysis");
        println!("");
        
        self.print_summary(graph);
        self.print_circular_dependencies(graph);
    }
    
    fn print_summary(&self, graph: &DependencyGraph) {
        style::print_subheading("Summary");
        println!("{}", style::format_key_value("Total Files", &graph.get_nodes().len().to_string()));
        
        let node_count = graph.get_nodes().len();
        
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
            
            println!("{}", style::format_key_value("Average Dependencies", &format!("{:.2}", total_dependencies as f64 / node_count as f64)));
            
            if !most_dependent_node.is_empty() {
                println!("{}", style::format_key_value("Most Dependent File", &most_dependent_node));
                println!("{}", style::format_key_value("Dependencies Count", &max_dependencies.to_string()));
            }
            
            if !most_depended_node.is_empty() {
                println!("{}", style::format_key_value("Most Depended-on File", &most_depended_node));
                println!("{}", style::format_key_value("Dependents Count", &max_dependents.to_string()));
            }
        }
        
        println!("");
    }
    
    fn print_circular_dependencies(&self, graph: &DependencyGraph) {
        let cycles = graph.find_circular_dependencies();
        
        style::print_subheading("Circular Dependencies");
        
        if cycles.is_empty() {
            println!("{}", style::format_info("No circular dependencies found."));
        } else {
            println!("{}", style::format_warning(&format!("Found {} circular dependencies:", cycles.len())));
            
            for (i, cycle) in cycles.iter().enumerate() {
                println!("{}", style::format_warning(&format!("Cycle {}: ", i + 1)));
                
                for (j, node) in cycle.iter().enumerate() {
                    if j < cycle.len() - 1 {
                        println!("  {} → ", node);
                    } else {
                        println!("  {} → (back to start)", node);
                    }
                }
                println!("");
            }
        }
    }
    
    pub fn export_dot<P: AsRef<Path>>(&self, graph: &DependencyGraph, output_path: P) -> Result<(), String> {
        let dot_content = graph.to_dot_format();
        
        fs::write(output_path, dot_content).map_err(|e| format!("Failed to write DOT file: {}", e))
    }
}