use crate::dependency::dependency_graph::DependencyGraph;
use crate::util::file_filter::FileFilter;
use crate::util::parallel;
use crate::output::path;
use crate::output::style;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tracing::{info, error, warn};

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
    
    let valid_formats = vec!["dot", "plantuml", "mermaid", "c4", "svg"];
    let valid_detail_levels = vec!["high", "medium", "low"];
    
    if !valid_formats.contains(&format.as_str()) {
        error!(
            "Invalid diagram format: {}. Valid formats are: {}",
            format,
            valid_formats.join(", ")
        );
        return 1;
    }
    
    if !valid_detail_levels.contains(&detail.as_str()) {
        error!(
            "Invalid detail level: {}. Valid detail levels are: {}",
            detail,
            valid_detail_levels.join(", ")
        );
        return 1;
    }
    
    let exclude_tests = !include_tests;
    
    let use_parallel = !no_parallel;
    
    let mut dependency_graph = DependencyGraph::new();
    
    
    info!("Building dependency graph...");
    
    let path_str = path.to_string_lossy().to_string();
    let is_parallel = use_parallel;
    
    parallel::log_parallel_status(is_parallel);
    
    let source_files = match crate::util::file_filter::get_all_source_files(&path_str, is_parallel) {
        Ok(files) => files,
        Err(err) => {
            error!("Failed to scan directory: {}", err);
            return 1;
        }
    };
    
    info!("Found {} source files", source_files.len());
    
    for file in &source_files {
        let file_path = file.to_string_lossy().to_string();
        
        if exclude_tests && FileFilter::is_test_file(file) {
            continue;
        }
        
        dependency_graph.add_node(&file_path);
        
        let parent_dir = file.parent();
        if let Some(dir) = parent_dir {
            for other_file in &source_files {
                if file == other_file {
                    continue;
                }
                
                if exclude_tests && FileFilter::is_test_file(other_file) {
                    continue;
                }
                
                let other_path = other_file.to_string_lossy().to_string();
                
                if other_file.parent() == Some(dir) {
                    dependency_graph.add_node(&other_path);
                    
                    if file_path.len() > other_path.len() {
                        dependency_graph.add_edge(&file_path, &other_path);
                    }
                }
            }
        }
    }
    
    let mut all_dependencies = HashMap::new();
    
    for node in dependency_graph.get_nodes() {
        let dependencies = dependency_graph.get_dependencies(node);
        all_dependencies.insert(node.clone(), dependencies);
    }
    
    let diagram_content = match format.as_str() {
        "dot" => {
            generate_dot_diagram(&all_dependencies, &detail, group_by_module, focus.as_deref())
        },
        "plantuml" => {
            generate_plantuml_diagram(&all_dependencies, &detail, group_by_module, focus.as_deref())
        },
        "mermaid" => {
            generate_mermaid_diagram(&all_dependencies, &detail, group_by_module, focus.as_deref())
        },
        "c4" => {
            generate_c4_diagram(&all_dependencies, &detail, group_by_module, focus.as_deref())
        },
        "svg" => {
            generate_svg_diagram(&all_dependencies, &detail, group_by_module, focus.as_deref())
        },
        _ => {
            error!("Unsupported diagram format: {}", format);
            return 1;
        }
    };
    
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
        
        if let Some(parent) = output_file.parent() {
            if !parent.exists() {
                if let Err(err) = fs::create_dir_all(parent) {
                    error!("Failed to create output directory: {}", err);
                    return 1;
                }
            }
        }
        
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
        
        suggest_visualization(&format, &output_file);
    } else {
        println!("\n{}", diagram_content);
    }
    
    0
}

fn get_file_extension(format: &str) -> &str {
    match format {
        "dot" => "dot",
        "plantuml" => "puml",
        "mermaid" => "mmd",
        "c4" => "puml",
        "svg" => "svg",
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
    
    if group_by_module {
        let module_dependencies = group_dependencies_by_module(&dependencies, focus);
        
        for (module, deps) in module_dependencies.iter() {
            dot.push_str(&format!("  subgraph cluster_{} {{\n", sanitize_node_id(module)));
            dot.push_str(&format!("    label=\"{}\";\n", module));
            dot.push_str("    style=filled;\n");
            dot.push_str("    color=lightgrey;\n");
            
            for (from, _) in deps.iter() {
                let file_name = extract_file_name(from);
                dot.push_str(&format!("    \"{}\" [label=\"{}\"];\n", sanitize_node_id(from), file_name));
            }
            
            dot.push_str("  }\n");
        }
        
        for (_module, deps) in module_dependencies.iter() {
            for (from, to_list) in deps.iter() {
                for to in to_list {
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
        for (from, _) in dependencies.iter() {
            let file_name = extract_file_name(from);
            
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
        
        for (from, to_list) in dependencies.iter() {
            if let Some(focus_path) = focus {
                if !from.contains(focus_path) {
                    continue;
                }
            }
            
            for to in to_list {
                if let Some(focus_path) = focus {
                    if !to.contains(focus_path) {
                        continue;
                    }
                }
                
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
    
    if group_by_module {
        let module_dependencies = group_dependencies_by_module(&dependencies, focus);
        
        for (module, deps) in module_dependencies.iter() {
            puml.push_str(&format!("package \"{}\" {{\n", module));
            
            for (from, _) in deps.iter() {
                let file_name = extract_file_name(from);
                puml.push_str(&format!("  component \"{}\" as {}\n", file_name, sanitize_node_id(from)));
            }
            
            puml.push_str("}\n\n");
        }
        
        for (_, deps) in module_dependencies.iter() {
            for (from, to_list) in deps.iter() {
                for to in to_list {
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
        for (from, _) in dependencies.iter() {
            let file_name = extract_file_name(from);
            
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
        
        for (from, to_list) in dependencies.iter() {
            if let Some(focus_path) = focus {
                if !from.contains(focus_path) {
                    continue;
                }
            }
            
            for to in to_list {
                if let Some(focus_path) = focus {
                    if !to.contains(focus_path) {
                        continue;
                    }
                }
                
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
    
    if group_by_module {
        let module_dependencies = group_dependencies_by_module(&dependencies, focus);
        
        for (module, deps) in module_dependencies.iter() {
            mmd.push_str(&format!("  subgraph {}\n", module));
            
            for (from, _) in deps.iter() {
                let file_name = extract_file_name(from);
                mmd.push_str(&format!("    {}[\"{}\"]\n", sanitize_node_id(from), file_name));
            }
            
            mmd.push_str("  end\n");
        }
        
        for (_, deps) in module_dependencies.iter() {
            for (from, to_list) in deps.iter() {
                for to in to_list {
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
        for (from, _) in dependencies.iter() {
            let file_name = extract_file_name(from);
            
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
        
        for (from, to_list) in dependencies.iter() {
            if let Some(focus_path) = focus {
                if !from.contains(focus_path) {
                    continue;
                }
            }
            
            for to in to_list {
                if let Some(focus_path) = focus {
                    if !to.contains(focus_path) {
                        continue;
                    }
                }
                
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
    
    c4.push_str("System_Boundary(system, \"Codebase Architecture\") {\n");
    
    if group_by_module {
        let module_dependencies = group_dependencies_by_module(&dependencies, focus);
        
        for (module, deps) in module_dependencies.iter() {
            let module_id = sanitize_node_id(module);
            c4.push_str(&format!("  Container({}Cont, \"{}\") {{\n", module_id, module));
            
            for (from, _) in deps.iter() {
                let file_name = extract_file_name(from);
                let file_id = sanitize_node_id(from);
                c4.push_str(&format!("    Component({}, \"{}\", \"File\")\n", file_id, file_name));
            }
            
            c4.push_str("  }\n\n");
        }
        
        for (_, deps) in module_dependencies.iter() {
            for (from, to_list) in deps.iter() {
                for to in to_list {
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
        for (from, _) in dependencies.iter() {
            let file_name = extract_file_name(from);
            
            if let Some(focus_path) = focus {
                if !from.contains(focus_path) {
                    continue;
                }
            }
            
            let file_id = sanitize_node_id(from);
            c4.push_str(&format!("  Component({}, \"{}\", \"File\")\n", file_id, file_name));
        }
        
        c4.push_str("\n");
        
        for (from, to_list) in dependencies.iter() {
            if let Some(focus_path) = focus {
                if !from.contains(focus_path) {
                    continue;
                }
            }
            
            for to in to_list {
                if let Some(focus_path) = focus {
                    if !to.contains(focus_path) {
                        continue;
                    }
                }
                
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

fn generate_svg_diagram(
    dependencies: &HashMap<String, Vec<String>>,
    detail_level: &str,
    group_by_module: bool,
    focus: Option<&str>,
) -> String {
    let dot_content = generate_dot_diagram(dependencies, detail_level, group_by_module, focus);
    
    match Command::new("dot").arg("-V").output() {
        Ok(_) => {
            match convert_dot_to_svg(&dot_content) {
                Ok(svg) => svg,
                Err(err) => {
                    warn!("Failed to convert DOT diagram to SVG: {}", err);
                    format!(
                        "<!-- Failed to generate SVG: {} -->\n<svg width=\"500\" height=\"200\" xmlns=\"http://www.w3.org/2000/svg\">\n  <rect width=\"500\" height=\"200\" fill=\"#f8f8f8\" stroke=\"#ccc\" stroke-width=\"1\"/>\n  <text x=\"20\" y=\"40\" font-family=\"sans-serif\" font-size=\"16\" fill=\"#333\">Error: {}</text>\n  <text x=\"20\" y=\"70\" font-family=\"sans-serif\" font-size=\"14\" fill=\"#666\">Please ensure Graphviz is properly configured.</text>\n</svg>",
                        err, err
                    )
                }
            }
        },
        Err(_) => {
            warn!("Graphviz (dot) is not installed. Cannot generate SVG diagram.");
            let error_svg = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n") +
                "<svg width=\"500\" height=\"200\" xmlns=\"http://www.w3.org/2000/svg\">\n" +
                "  <rect width=\"500\" height=\"200\" fill=\"#f8f8f8\" stroke=\"#ccc\" stroke-width=\"1\"/>\n" +
                "  <text x=\"20\" y=\"40\" font-family=\"sans-serif\" font-size=\"16\" fill=\"#333\">Error: Graphviz (dot) is not installed</text>\n" +
                "  <text x=\"20\" y=\"70\" font-family=\"sans-serif\" font-size=\"14\" fill=\"#666\">To generate SVG diagrams:</text>\n" +
                "  <text x=\"20\" y=\"100\" font-family=\"sans-serif\" font-size=\"14\" fill=\"#666\">1. Install Graphviz from https://graphviz.org/download/</text>\n" +
                "  <text x=\"20\" y=\"130\" font-family=\"sans-serif\" font-size=\"14\" fill=\"#666\">2. Make sure 'dot' command is available in your PATH</text>\n" +
                "  <text x=\"20\" y=\"160\" font-family=\"sans-serif\" font-size=\"14\" fill=\"#666\">3. Run the command again</text>\n" +
                "</svg>";
            error_svg
        }
    }
}

fn convert_dot_to_svg(dot_content: &str) -> Result<String, String> {
    let mut process = match Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
            Ok(p) => p,
            Err(e) => return Err(format!("Failed to spawn Graphviz: {}", e)),
        };
    
    if let Some(mut stdin) = process.stdin.take() {
        if let Err(e) = stdin.write_all(dot_content.as_bytes()) {
            return Err(format!("Failed to write to Graphviz stdin: {}", e));
        }
    }
    
    match process.wait_with_output() {
        Ok(output) => {
            if output.status.success() {
                match String::from_utf8(output.stdout) {
                    Ok(svg) => Ok(svg),
                    Err(e) => Err(format!("Invalid UTF-8 in SVG output: {}", e)),
                }
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("Graphviz error: {}", error))
            }
        },
        Err(e) => Err(format!("Failed to get Graphviz output: {}", e)),
    }
}

fn group_dependencies_by_module(
    dependencies: &HashMap<String, Vec<String>>,
    focus: Option<&str>,
) -> HashMap<String, HashMap<String, Vec<String>>> {
    let mut module_deps: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
    
    for (from, to_list) in dependencies {
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

fn extract_module_name(file_path: &str) -> String {
    let path = Path::new(file_path);
    
    if let Some(parent) = path.parent() {
        if let Some(file_name) = parent.file_name() {
            if let Some(name) = file_name.to_str() {
                return name.to_string();
            }
        }
    }
    
    "unknown".to_string()
}

fn extract_file_name(file_path: &str) -> String {
    let path = Path::new(file_path);
    
    if let Some(file_name) = path.file_name() {
        if let Some(name) = file_name.to_str() {
            return name.to_string();
        }
    }
    
    file_path.to_string()
}

fn sanitize_node_id(id: &str) -> String {
    id.replace(&['.', '/', '\\', ' ', '-'][..], "_")
}

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
        "svg" => {
            println!("To view the SVG diagram:");
            println!("1. Open the file in any web browser or SVG viewer");
            println!("2. To import into LucidChart:");
            println!("   - Open LucidChart and create a new diagram");
            println!("   - Click on File → Import → SVG");
            println!("   - Select the generated SVG file ({})", output_file.display());
            
            if let Ok(content) = fs::read_to_string(output_file) {
                if content.contains("<!-- Failed to generate SVG") {
                    println!("\nNOTE: The SVG generation appears to have encountered an error.");
                    println!("Please ensure Graphviz is installed on your system:");
                    println!("- For MacOS: brew install graphviz");
                    println!("- For Ubuntu/Debian: sudo apt-get install graphviz");
                    println!("- For Windows: winget install graphviz");
                }
            }
        }
        _ => {}
    }
}
