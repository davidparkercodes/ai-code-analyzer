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
        true, // no_parallel = true
        true, // no_git = true
        true, // force = true
        false // dry_run = false
    );
    
    // Verify success
    assert_eq!(exit_code, 0);
    
    // Check if comments were removed correctly
    let fn1 = "test1".to_string() + ".rs";
    let cleaned_file1 = fs::read_to_string(
        Path::new(&output_path).join(&fn1)
    ).unwrap();
    
    let fn2 = "test2".to_string() + ".rs";
    let cleaned_file2 = fs::read_to_string(
        Path::new(&output_path).join(&fn2)
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
    assert!(cleaned_file2.contains("// aicodeanalyzer: ignore"));
    
    // Check file 3 with ignore pattern
    let fn3 = "test3".to_string() + ".rs";
    let cleaned_file3 = fs::read_to_string(
        Path::new(&output_path).join(&fn3)
    ).unwrap();
    
    assert!(!cleaned_file3.contains("// This comment will be removed"));
    assert!(cleaned_file3.contains("// aicodeanalyzer: ignore"));
    assert!(!cleaned_file3.contains("// This comment will be removed"));
    
    // Check file 4 with string literals containing comment-like text
    let fn4 = "test4".to_string() + ".rs";
    let cleaned_file4 = fs::read_to_string(
        Path::new(&output_path).join(&fn4)
    ).unwrap();
    
    // Verify real comments are removed
    assert!(!cleaned_file4.contains("// Real comment"));
    
    // Verify string literals with comment-like text are preserved
    assert!(cleaned_file4.contains("This string contains // a comment-like pattern"));
    assert!(cleaned_file4.contains("Anthropic Claude"));
    assert!(cleaned_file4.contains("Multiple // comment // patterns"));
    
    // Verify raw string literals are preserved
    assert!(cleaned_file4.contains("raw string with // comment pattern"));
    
    // Verify strings with escape sequences are preserved
    assert!(cleaned_file4.contains("Escaped quote"));
    
    // Verify format strings are preserved
    assert!(cleaned_file4.contains("Generate code for task:"));
}

#[test]
fn test_dry_run_mode() {
    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create test Rust files with comments
    create_test_files(temp_path);
    
    // Path to first test file
    let fn1 = "test1".to_string() + ".rs";
    let test_file_path = temp_path.join(&fn1);
    let original_content = fs::read_to_string(&test_file_path).unwrap();
    
    // Run the clean_comments command with dry_run = true
    let exit_code = clean_comments::execute(
        test_file_path.to_str().unwrap().to_string(),
        None, // No output directory, would modify in-place if not dry run
        true, // no_parallel = true
        true, // no_git = true
        true, // force = true
        true  // dry_run = true
    );
    
    // Verify success
    assert_eq!(exit_code, 0);
    
    // Verify the file was NOT modified (since we're in dry-run mode)
    let content_after_run = fs::read_to_string(&test_file_path).unwrap();
    assert_eq!(original_content, content_after_run);
    assert!(content_after_run.contains("// This is a comment"));
    assert!(content_after_run.contains("// End of line comment"));
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
    name: String, // aicodeanalyzer: ignore
}
"#;
    
    // Test file 3 with ignore pattern
    let file3_content = r#"// This comment will be removed
fn test() {
    // aicodeanalyzer: ignore
    let y = 10; // This comment will be removed
}
"#;

    // Test file 4 with string literals containing comment-like text
    let file4_content = r#"
fn string_literals_with_comments() {
    // Real comment
    let str1 = "This string contains // a comment-like pattern";
    let str2 = "Anthropic Claude";
    let str3 = "Multiple // comment // patterns";
    
    // Test raw strings
    let raw_str = r##"This is a raw string with // comment pattern"##;
    
    // Test with escape sequences
    let escaped_str = "Escaped quote \\\" and // comment";
    
    // Test simple format
    let formatted = format!("Generate code for task: {}", "task");
}
"#;
    
    // Compose filenames to avoid issues with string literals
    let fn1 = "test1".to_string() + ".rs";
    let fn2 = "test2".to_string() + ".rs";
    let fn3 = "test3".to_string() + ".rs";
    let fn4 = "test4".to_string() + ".rs";
    
    let test1_path = dir_path.join(&fn1);
    let test2_path = dir_path.join(&fn2);
    let test3_path = dir_path.join(&fn3);
    let test4_path = dir_path.join(&fn4);
    
    fs::write(&test1_path, file1_content).unwrap();
    fs::write(&test2_path, file2_content).unwrap();
    fs::write(&test3_path, file3_content).unwrap();
    fs::write(&test4_path, file4_content).unwrap();
}