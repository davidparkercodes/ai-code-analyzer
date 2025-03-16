use ai_code_analyzer::commands::architecture_diagram;
use tempfile::TempDir;
use std::path::Path;
use std::fs;

#[tokio::test]
async fn test_architecture_diagram_command() {
    // Create a temporary directory with a simple codebase structure
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap().to_string();
    
    // Create a src directory
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    
    // Create some simple Rust files with dependencies
    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, "mod lib;\nfn main() {\n    lib::do_something();\n}\n").unwrap();
    
    let lib_rs = src_dir.join("lib.rs");
    fs::write(&lib_rs, "mod utils;\npub fn do_something() {\n    utils::helper();\n}\n").unwrap();
    
    // Create a utils directory and module
    let utils_dir = src_dir.join("utils");
    fs::create_dir_all(&utils_dir).unwrap();
    
    let mod_rs = utils_dir.join("mod.rs");
    fs::write(&mod_rs, "pub fn helper() {\n    println!(\"Helper function\");\n}\n").unwrap();
    
    // Run the architecture-diagram command with dot format
    let result = architecture_diagram::execute(
        temp_path.clone(),
        true, // no_output (don't save to file)
        None, // output_path
        true, // no_parallel
        "dot".to_string(), // format
        "medium".to_string(), // detail
        false, // include_tests
        false, // group_by_module
        None, // focus
    ).await;
    
    // Verify the command succeeded
    assert_eq!(result, 0);
    
    // Run with different format
    let result_plantuml = architecture_diagram::execute(
        temp_path.clone(),
        true, // no_output (don't save to file)
        None, // output_path
        true, // no_parallel
        "plantuml".to_string(), // format
        "medium".to_string(), // detail
        false, // include_tests
        true, // group_by_module
        None, // focus
    ).await;
    
    // Verify the command succeeded
    assert_eq!(result_plantuml, 0);
    
    // Run with invalid format (should fail)
    let result_invalid = architecture_diagram::execute(
        temp_path.clone(),
        true, // no_output (don't save to file)
        None, // output_path
        true, // no_parallel
        "invalid".to_string(), // format
        "medium".to_string(), // detail
        false, // include_tests
        false, // group_by_module
        None, // focus
    ).await;
    
    // Verify the command failed due to invalid format
    assert_eq!(result_invalid, 1);
    
    // Test with output file
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();
    let output_path = output_dir.join("arch.dot").to_str().unwrap().to_string();
    
    let result_output = architecture_diagram::execute(
        temp_path.clone(),
        false, // no_output
        Some(output_path.clone()), // output_path
        true, // no_parallel
        "dot".to_string(), // format
        "medium".to_string(), // detail
        false, // include_tests
        false, // group_by_module
        None, // focus
    ).await;
    
    // Verify the command succeeded
    assert_eq!(result_output, 0);
    
    // Verify the output file exists
    assert!(Path::new(&output_path).exists());
    
    // Verify the output file contains DOT graph content
    let dot_content = fs::read_to_string(&output_path).unwrap();
    assert!(dot_content.starts_with("digraph ArchitectureDiagram {"));
    assert!(dot_content.contains("rankdir=LR;"));
}