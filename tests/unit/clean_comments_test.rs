use std::fs;
use std::path::Path;
use tempfile::TempDir;
use code_analyzer::commands::clean_comments;

#[test]
fn test_clean_comments_from_rust_files() {
    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create test Rust files with comments
    create_test_files(temp_path);
    
    // Create output directory
    let output_dir = TempDir::new().unwrap();
    let output_path = output_dir.path().to_str().unwrap().to_string();
    
    // Run the clean_comments command
    let exit_code = clean_comments::execute(
        temp_path.to_str().unwrap().to_string(),
        Some(output_path.clone()),
        true // no_parallel = true
    );
    
    // Verify success
    assert_eq!(exit_code, 0);
    
    // Check if comments were removed correctly
    let cleaned_file1 = fs::read_to_string(
        Path::new(&output_path).join("test1.rs")
    ).unwrap();
    
    let cleaned_file2 = fs::read_to_string(
        Path::new(&output_path).join("test2.rs")
    ).unwrap();
    
    // Verify file 1
    assert!(!cleaned_file1.contains("// This is a comment"));
    assert!(cleaned_file1.contains("fn main() {"));
    assert!(cleaned_file1.contains("let x = 5;"));
    assert!(!cleaned_file1.contains("// End of line comment"));
    
    // Verify file 2
    assert!(!cleaned_file2.contains("// Another comment"));
    assert!(cleaned_file2.contains("struct Test {"));
    assert!(cleaned_file2.contains("value: i32,"));
    assert!(cleaned_file2.contains("/// Doc comment should remain"));
    assert!(!cleaned_file2.contains("// This should be removed"));
}

fn create_test_files(dir_path: &Path) {
    // Test file 1
    let file1_content = r#"// This is a comment
fn main() {
    let x = 5; // End of line comment
    println!("Hello");
}
"#;
    
    // Test file 2
    let file2_content = r#"// Another comment
struct Test {
    /// Doc comment should remain
    value: i32, // This should be removed
}
"#;
    
    fs::write(dir_path.join("test1.rs"), file1_content).unwrap();
    fs::write(dir_path.join("test2.rs"), file2_content).unwrap();
}