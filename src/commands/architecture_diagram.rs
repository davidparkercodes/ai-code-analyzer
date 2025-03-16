use crate::dependency::dependency_graph::DependencyGraph;
use crate::util::file_filter::FileFilter;
use crate::util::parallel;
use crate::output::path;
use crate::output::style;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use tracing::{info, error};

pub async fn execute(
    path: String, 
    no_output: bool, 
    output_path: Option<String>, 
    no_parallel: bool,
    format: String,
    detail: String,
    include_tests: bool,
    group_by_module: bool,
    focus: Option<String>,
) -> i32 {
    let path = Path::new(&path);
    
    if !path.exists() {
        error!("Path does not exist: {}", path.display());
        return 1;
    }
    
    info!("Analyzing architecture for path: {}", path.display());
    info!("Diagram format: {}", format);
    info!("Diagram detail level: {}", detail);
    
    // Define valid formats and detail levels
    let valid_formats = vec!["dot", "plantuml", "mermaid", "c4"];
    let valid_detail_levels = vec!["high", "medium", "low"];
    
    // Validate format
    if !valid_formats.contains(&format.as_str()) {
        error!(
            "Invalid diagram format: {}. Valid formats are: {}",
            format,
            valid_formats.join(", ")
        );
        return 1;
    }
    
    // Validate detail level
    if !valid_detail_levels.contains(&detail.as_str()) {
        error!(
            "Invalid detail level: {}. Valid detail levels are: {}",
            detail,
            valid_detail_levels.join(", ")
        );
        return 1;
    }
    
    // Configure file filter settings
    let exclude_tests = !include_tests;
    
    // Configure parallel processing
    let use_parallel = !no_parallel;
    
    // Create a dependency graph to analyze relationships
    let mut dependency_graph = DependencyGraph::new();
    
    // Build dependency graph using the existing analyzer infrastructure
    // For this example, we'll create a simplified graph for demo purposes
    // In a real implementation, this would integrate with the actual dependency analyzer
    
    info!("Building dependency graph...");
    
    let path_str = path.to_string_lossy().to_string();
    let is_parallel = use_parallel;
    
    // Log parallel processing status
    parallel::log_parallel_status(is_parallel);
    
    // Use the existing file utilities to gather source files with the right filters
    let source_files = match crate::util::file_filter::get_all_source_files(&path_str, is_parallel) {
        Ok(files) => files,
        Err(err) => {
            error!("Failed to scan directory: {}", err);
            return 1;
        }
    };
    
    info!("Found {} source files", source_files.len());
    
    // Create a naive dependency map based on directory structure for demo
    // This is a placeholder - the real implementation would use more sophisticated dependency analysis
    for file in &source_files {
        let file_path = file.to_string_lossy().to_string();
        
        // Skip test files if excluded
        if exclude_tests && FileFilter::is_test_file(file) {
            continue;
        }
        
        // Add the file as a node
        dependency_graph.add_node(&file_path);
        
        // Add edges to files in the same directory (simplified demo)
        let parent_dir = file.parent();
        if let Some(dir) = parent_dir {
            for other_file in &source_files {
                if file == other_file {
                    continue;
                }
                
                // Skip test files if excluded
                if exclude_tests && FileFilter::is_test_file(other_file) {
                    continue;
                }
                
                let other_path = other_file.to_string_lossy().to_string();
                
                if other_file.parent() == Some(dir) {
                    // Add the other file as a node
                    dependency_graph.add_node(&other_path);
                    
                    // Add an edge from this file to the other (arbitrary for demo)
                    // In a real implementation, we would analyze imports/includes
                    if file_path.len() > other_path.len() {
                        dependency_graph.add_edge(&file_path, &other_path);
                    }
                }
            }
        }
    }
    
    // Create the diagram based on format
    let diagram_content = match format.as_str() {
        "dot" => {
            // Create a map of all dependencies for diagram generation
            let mut all_dependencies = HashMap::new();
            
            for node in dependency_graph.get_nodes() {
                let dependencies = dependency_graph.get_dependencies(node);
                all_dependencies.insert(node.clone(), dependencies);
            }
            
            generate_dot_diagram(&all_dependencies, &detail, group_by_module, focus.as_deref())
        },
        "plantuml" => {
            // Create a map of all dependencies for diagram generation
            let mut all_dependencies = HashMap::new();
            
            for node in dependency_graph.get_nodes() {
                let dependencies = dependency_graph.get_dependencies(node);
                all_dependencies.insert(node.clone(), dependencies);
            }
            
            generate_plantuml_diagram(&all_dependencies, &detail, group_by_module, focus.as_deref())
        },
        "mermaid" => {
            // Create a map of all dependencies for diagram generation
            let mut all_dependencies = HashMap::new();
            
            for node in dependency_graph.get_nodes() {
                let dependencies = dependency_graph.get_dependencies(node);
                all_dependencies.insert(node.clone(), dependencies);
            }
            
            generate_mermaid_diagram(&all_dependencies, &detail, group_by_module, focus.as_deref())
        },
        "c4" => {
            // Create a map of all dependencies for diagram generation
            let mut all_dependencies = HashMap::new();
            
            for node in dependency_graph.get_nodes() {
                let dependencies = dependency_graph.get_dependencies(node);
                all_dependencies.insert(node.clone(), dependencies);
            }
            
            generate_c4_diagram(&all_dependencies, &detail, group_by_module, focus.as_deref())
        },
        _ => {
            error!("Unsupported diagram format: {}", format);
            return 1;
        }
    };
    
    // Handle output
    if !no_output {
        let output_file = match output_path {
            Some(custom_path) => PathBuf::from(custom_path),
            None => {
                let dir_name = if path == Path::new(".") {
                    std::env::current_dir()
                        .ok()
                        .and_then(|p| p.file_name().and_then(|n| n.to_str().map(String::from)))
                        .unwrap_or_else(|| "current_dir".to_string())
                } else {
                    path.file_name()
                        .and_then(|n| n.to_str().map(String::from))
                        .unwrap_or_else(|| "unknown".to_string())
                };
                
                let output_file = match path::resolve_output_path(
                    "architecture-diagram",
                    &dir_name,
                    get_file_extension(&format),
                ) {
                    Ok(p) => p,
                    Err(err) => {
                        error!("Failed to create output path: {}", err);
                        return 1;
                    }
                };
                
                output_file
            }
        };
        
        // Create directory if it doesn't exist
        if let Some(parent) = output_file.parent() {
            if !parent.exists() {
                if let Err(err) = fs::create_dir_all(parent) {
                    error!("Failed to create output directory: {}", err);
                    return 1;
                }
            }
        }
        
        // Write diagram to file
        if let Err(err) = fs::write(&output_file, diagram_content) {
            error!("Failed to write diagram to file: {}", err);
            return 1;
        }
        
        println!(
            "{}",
            style::success(&format!(
                "Architecture diagram saved to: {}",
                output_file.display()
            ))
        );
        
        // Suggest visualizing the diagram based on format
        suggest_visualization(&format, &output_file);
    } else {
        // Print diagram to console
        println!("\n{}", diagram_content);
    }
    
    // Success
    0
}

fn get_file_extension(format: &str) -> &str {
    match format {
        "dot" => "dot",
        "plantuml" => "puml",
        "mermaid" => "mmd",
        "c4" => "puml",
        _ => "txt",
    }
}

fn generate_dot_diagram(
    dependencies: &HashMap<String, Vec<String>>,
    detail_level: &str,
    group_by_module: bool,
    focus: Option<&str>,
) -> String {
    let mut dot = String::from("digraph ArchitectureDiagram {\n");
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  node [shape=box, style=filled, fillcolor=lightblue];\n");
    dot.push_str("  edge [arrowhead=vee];\n\n");
    
    // Process nodes and edges based on the grouping option
    if group_by_module {
        // Group nodes by module
        let module_dependencies = group_dependencies_by_module(&dependencies, focus);
        
        // Create module subgraphs
        for (module, deps) in module_dependencies.iter() {
            dot.push_str(&format!("  subgraph cluster_{} {{\n", sanitize_node_id(module)));
            dot.push_str(&format!("    label=\"{}\";\n", module));
            dot.push_str("    style=filled;\n");
            dot.push_str("    color=lightgrey;\n");
            
            // Add module files as nodes in the subgraph
            for (from, _) in deps.iter() {
                let file_name = extract_file_name(from);
                dot.push_str(&format!("    \"{}\" [label=\"{}\"];\n", sanitize_node_id(from), file_name));
            }
            
            dot.push_str("  }\n");
        }
        
        // Add edges between nodes
        for (module, deps) in module_dependencies.iter() {
            for (from, to_list) in deps.iter() {
                for to in to_list {
                    // Skip self-references if detail level is low
                    if detail_level == "low" && from == to {
                        continue;
                    }
                    
                    dot.push_str(&format!(
                        "  \"{}\" -> \"{}\";\n",
                        sanitize_node_id(from),
                        sanitize_node_id(to)
                    ));
                }
            }
        }
    } else {
        // Add nodes
        for (from, to_list) in dependencies.iter() {
            let file_name = extract_file_name(from);
            
            // Skip if not in focus
            if let Some(focus_path) = focus {
                if !from.contains(focus_path) {
                    continue;
                }
            }
            
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\"];\n",
                sanitize_node_id(from),
                file_name
            ));
        }
        
        // Add edges
        for (from, to_list) in dependencies.iter() {
            // Skip if not in focus
            if let Some(focus_path) = focus {
                if !from.contains(focus_path) {
                    continue;
                }
            }
            
            for to in to_list {
                // Skip if target is not in focus
                if let Some(focus_path) = focus {
                    if !to.contains(focus_path) {
                        continue;
                    }
                }
                
                // Skip self-references if detail level is low
                if detail_level == "low" && from == to {
                    continue;
                }
                
                dot.push_str(&format!(
                    "  \"{}\" -> \"{}\";\n",
                    sanitize_node_id(from),
                    sanitize_node_id(to)
                ));
            }
        }
    }
    
    dot.push_str("}\n");
    dot
}

fn generate_plantuml_diagram(
    dependencies: &HashMap<String, Vec<String>>,
    detail_level: &str,
    group_by_module: bool,
    focus: Option<&str>,
) -> String {
    let mut puml = String::from("@startuml Architecture\n");
    puml.push_str("!theme sketchy-outline\n");
    puml.push_str("skinparam linetype ortho\n\n");
    
    // Process nodes and edges based on the grouping option
    if group_by_module {
        // Group nodes by module
        let module_dependencies = group_dependencies_by_module(&dependencies, focus);
        
        // Create modules as packages
        for (module, deps) in module_dependencies.iter() {
            puml.push_str(&format!("package \"{}\" {{\n", module));
            
            // Add module files as components
            for (from, _) in deps.iter() {
                let file_name = extract_file_name(from);
                puml.push_str(&format!("  component \"{}\" as {}\n", file_name, sanitize_node_id(from)));
            }
            
            puml.push_str("}\n\n");
        }
        
        // Add relationships between components
        for (_, deps) in module_dependencies.iter() {
            for (from, to_list) in deps.iter() {
                for to in to_list {
                    // Skip self-references if detail level is low
                    if detail_level == "low" && from == to {
                        continue;
                    }
                    
                    puml.push_str(&format!(
                        "{} --> {}\n",
                        sanitize_node_id(from),
                        sanitize_node_id(to)
                    ));
                }
            }
        }
    } else {
        // Add components
        for (from, _) in dependencies.iter() {
            let file_name = extract_file_name(from);
            
            // Skip if not in focus
            if let Some(focus_path) = focus {
                if !from.contains(focus_path) {
                    continue;
                }
            }
            
            puml.push_str(&format!(
                "component \"{}\" as {}\n",
                file_name,
                sanitize_node_id(from)
            ));
        }
        
        puml.push_str("\n");
        
        // Add relationships
        for (from, to_list) in dependencies.iter() {
            // Skip if not in focus
            if let Some(focus_path) = focus {
                if !from.contains(focus_path) {
                    continue;
                }
            }
            
            for to in to_list {
                // Skip if target is not in focus
                if let Some(focus_path) = focus {
                    if !to.contains(focus_path) {
                        continue;
                    }
                }
                
                // Skip self-references if detail level is low
                if detail_level == "low" && from == to {
                    continue;
                }
                
                puml.push_str(&format!(
                    "{} --> {}\n",
                    sanitize_node_id(from),
                    sanitize_node_id(to)
                ));
            }
        }
    }
    
    puml.push_str("@enduml\n");
    puml
}

fn generate_mermaid_diagram(
    dependencies: &HashMap<String, Vec<String>>,
    detail_level: &str,
    group_by_module: bool,
    focus: Option<&str>,
) -> String {
    let mut mmd = String::from("graph LR\n");
    
    // Process nodes and edges based on the grouping option
    if group_by_module {
        // Group nodes by module
        let module_dependencies = group_dependencies_by_module(&dependencies, focus);
        
        // Create module subgraphs
        for (module, deps) in module_dependencies.iter() {
            mmd.push_str(&format!("  subgraph {}\n", module));
            
            // Add module files as nodes in the subgraph
            for (from, _) in deps.iter() {
                let file_name = extract_file_name(from);
                mmd.push_str(&format!("    {}[\"{}\"]\n", sanitize_node_id(from), file_name));
            }
            
            mmd.push_str("  end\n");
        }
        
        // Add edges between nodes
        for (_, deps) in module_dependencies.iter() {
            for (from, to_list) in deps.iter() {
                for to in to_list {
                    // Skip self-references if detail level is low
                    if detail_level == "low" && from == to {
                        continue;
                    }
                    
                    mmd.push_str(&format!(
                        "  {} --> {}\n",
                        sanitize_node_id(from),
                        sanitize_node_id(to)
                    ));
                }
            }
        }
    } else {
        // Add nodes
        for (from, _) in dependencies.iter() {
            let file_name = extract_file_name(from);
            
            // Skip if not in focus
            if let Some(focus_path) = focus {
                if !from.contains(focus_path) {
                    continue;
                }
            }
            
            mmd.push_str(&format!(
                "  {}[\"{}\"]\n",
                sanitize_node_id(from),
                file_name
            ));
        }
        
        // Add edges
        for (from, to_list) in dependencies.iter() {
            // Skip if not in focus
            if let Some(focus_path) = focus {
                if !from.contains(focus_path) {
                    continue;
                }
            }
            
            for to in to_list {
                // Skip if target is not in focus
                if let Some(focus_path) = focus {
                    if !to.contains(focus_path) {
                        continue;
                    }
                }
                
                // Skip self-references if detail level is low
                if detail_level == "low" && from == to {
                    continue;
                }
                
                mmd.push_str(&format!(
                    "  {} --> {}\n",
                    sanitize_node_id(from),
                    sanitize_node_id(to)
                ));
            }
        }
    }
    
    mmd
}

fn generate_c4_diagram(
    dependencies: &HashMap<String, Vec<String>>,
    detail_level: &str,
    group_by_module: bool,
    focus: Option<&str>,
) -> String {
    let mut c4 = String::from("@startuml C4_Architecture\n");
    c4.push_str("!includeurl https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml\n\n");
    c4.push_str("LAYOUT_WITH_LEGEND()\n\n");
    
    // Generate system context
    c4.push_str("System_Boundary(system, \"Codebase Architecture\") {\n");
    
    // Process nodes and edges based on the grouping option
    if group_by_module {
        // Group nodes by module
        let module_dependencies = group_dependencies_by_module(&dependencies, focus);
        
        // Create modules as containers
        for (module, deps) in module_dependencies.iter() {
            let module_id = sanitize_node_id(module);
            c4.push_str(&format!("  Container({}Cont, \"{}\") {{\n", module_id, module));
            
            // Add module files as components
            for (from, _) in deps.iter() {
                let file_name = extract_file_name(from);
                let file_id = sanitize_node_id(from);
                c4.push_str(&format!("    Component({}, \"{}\", \"File\")\n", file_id, file_name));
            }
            
            c4.push_str("  }\n\n");
        }
        
        // Add relationships between components
        for (_, deps) in module_dependencies.iter() {
            for (from, to_list) in deps.iter() {
                for to in to_list {
                    // Skip self-references if detail level is low
                    if detail_level == "low" && from == to {
                        continue;
                    }
                    
                    let from_id = sanitize_node_id(from);
                    let to_id = sanitize_node_id(to);
                    c4.push_str(&format!("  Rel({}, {}, \"depends on\")\n", from_id, to_id));
                }
            }
        }
    } else {
        // Add components
        for (from, _) in dependencies.iter() {
            let file_name = extract_file_name(from);
            
            // Skip if not in focus
            if let Some(focus_path) = focus {
                if !from.contains(focus_path) {
                    continue;
                }
            }
            
            let file_id = sanitize_node_id(from);
            c4.push_str(&format!("  Component({}, \"{}\", \"File\")\n", file_id, file_name));
        }
        
        c4.push_str("\n");
        
        // Add relationships
        for (from, to_list) in dependencies.iter() {
            // Skip if not in focus
            if let Some(focus_path) = focus {
                if !from.contains(focus_path) {
                    continue;
                }
            }
            
            for to in to_list {
                // Skip if target is not in focus
                if let Some(focus_path) = focus {
                    if !to.contains(focus_path) {
                        continue;
                    }
                }
                
                // Skip self-references if detail level is low
                if detail_level == "low" && from == to {
                    continue;
                }
                
                let from_id = sanitize_node_id(from);
                let to_id = sanitize_node_id(to);
                c4.push_str(&format!("  Rel({}, {}, \"depends on\")\n", from_id, to_id));
            }
        }
    }
    
    c4.push_str("}\n");
    c4.push_str("@enduml\n");
    c4
}

// Group dependencies by module path
fn group_dependencies_by_module(
    dependencies: &HashMap<String, Vec<String>>,
    focus: Option<&str>,
) -> HashMap<String, HashMap<String, Vec<String>>> {
    let mut module_deps: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
    
    for (from, to_list) in dependencies {
        // Skip if not in focus
        if let Some(focus_path) = focus {
            if !from.contains(focus_path) {
                continue;
            }
        }
        
        let module_name = extract_module_name(from);
        let module_entry = module_deps.entry(module_name).or_insert_with(HashMap::new);
        
        let filtered_to_list: Vec<String> = to_list
            .iter()
            .filter(|to| {
                if let Some(focus_path) = focus {
                    to.contains(focus_path)
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        
        if !filtered_to_list.is_empty() {
            module_entry.insert(from.clone(), filtered_to_list);
        }
    }
    
    module_deps
}

// Extract module name from file path
fn extract_module_name(file_path: &str) -> String {
    let path = Path::new(file_path);
    
    // Get parent directory name as module
    if let Some(parent) = path.parent() {
        if let Some(file_name) = parent.file_name() {
            if let Some(name) = file_name.to_str() {
                return name.to_string();
            }
        }
    }
    
    // Default module name if extraction fails
    "unknown".to_string()
}

// Extract file name from path
fn extract_file_name(file_path: &str) -> String {
    let path = Path::new(file_path);
    
    if let Some(file_name) = path.file_name() {
        if let Some(name) = file_name.to_str() {
            return name.to_string();
        }
    }
    
    // Return path as fallback
    file_path.to_string()
}

// Sanitize node id for diagram compatibility
fn sanitize_node_id(id: &str) -> String {
    id.replace(&['.', '/', '\\', ' ', '-'][..], "_")
}

// Suggest visualization methods based on diagram format
fn suggest_visualization(format: &str, output_file: &Path) {
    match format {
        "dot" => {
            println!("To visualize the DOT diagram:");
            println!("1. Install Graphviz (if not already installed)");
            println!("2. Run: dot -Tpng {} -o architecture.png", output_file.display());
        }
        "plantuml" | "c4" => {
            println!("To visualize the PlantUML diagram:");
            println!("1. Use an online PlantUML server like http://www.plantuml.com/plantuml/");
            println!("2. Or install PlantUML locally: https://plantuml.com/download");
            println!("3. Run: java -jar plantuml.jar {}", output_file.display());
        }
        "mermaid" => {
            println!("To visualize the Mermaid diagram:");
            println!("1. Use the Mermaid Live Editor: https://mermaid.live/");
            println!("2. Or install Mermaid CLI: npm install -g @mermaid-js/mermaid-cli");
            println!("3. Run: mmdc -i {} -o architecture.png", output_file.display());
        }
        _ => {}
    }
}