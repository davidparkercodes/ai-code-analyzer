use std::fs;
use std::path::Path;
use tempfile::TempDir;
use code_analyzer::commands::delete_comments;

#[test]
fn test_delete_comments_from_rust_files() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    create_test_files(temp_path);
    
    let output_dir = TempDir::new().unwrap();
    let output_path = output_dir.path().to_str().unwrap().to_string();
    
    let exit_code = delete_comments::execute(
        temp_path.to_str().unwrap().to_string(),
        "rust",
        false,
        Some(output_path.clone()),
        true,
        true,
        true,
        false
    );
    
    assert_eq!(exit_code, 0);
    
    let fn1 = "test1".to_string() + ".rs";
    let cleaned_file1 = fs::read_to_string(
        Path::new(&output_path).join(&fn1)
    ).unwrap();
    
    let fn2 = "test2".to_string() + ".rs";
    let cleaned_file2 = fs::read_to_string(
        Path::new(&output_path).join(&fn2)
    ).unwrap();
    
    assert!(!cleaned_file1.contains("// This is a comment"));
    assert!(cleaned_file1.contains("fn main() {"));
    assert!(cleaned_file1.contains("let x = 5;"));
    assert!(!cleaned_file1.contains("// End of line comment"));
    
    assert!(!cleaned_file2.contains("// Another comment"));
    assert!(cleaned_file2.contains("struct Test {"));
    assert!(cleaned_file2.contains("value: i32,"));
    assert!(cleaned_file2.contains("/// Doc comment should remain"));
    assert!(!cleaned_file2.contains("// This should be removed"));
    assert!(cleaned_file2.contains("// aicodeanalyzer: ignore"));
    
    let fn3 = "test3".to_string() + ".rs";
    let cleaned_file3 = fs::read_to_string(
        Path::new(&output_path).join(&fn3)
    ).unwrap();
    
    assert!(!cleaned_file3.contains("// This comment will be removed"));
    assert!(cleaned_file3.contains("// aicodeanalyzer: ignore"));
    assert!(!cleaned_file3.contains("// This comment will be removed"));
    
    let fn4 = "test4".to_string() + ".rs";
    let cleaned_file4 = fs::read_to_string(
        Path::new(&output_path).join(&fn4)
    ).unwrap();
    
    assert!(!cleaned_file4.contains("// Real comment"));
    
    assert!(cleaned_file4.contains("This string contains // a comment-like pattern"));
    assert!(cleaned_file4.contains("Anthropic Claude"));
    assert!(cleaned_file4.contains("Multiple // comment // patterns"));
    
    assert!(cleaned_file4.contains("raw string with // comment pattern"));
    
    assert!(cleaned_file4.contains("Escaped quote"));
    
    assert!(cleaned_file4.contains("Generate code for task:"));
}

#[test]
fn test_dry_run_mode() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    create_test_files(temp_path);
    
    let fn1 = "test1".to_string() + ".rs";
    let test_file_path = temp_path.join(&fn1);
    let original_content = fs::read_to_string(&test_file_path).unwrap();
    
    let exit_code = delete_comments::execute(
        test_file_path.to_str().unwrap().to_string(),
        "rust",
        true,
        None,
        true,
        true,
        true,
        true
    );
    
    assert_eq!(exit_code, 0);
    
    let content_after_run = fs::read_to_string(&test_file_path).unwrap();
    assert_eq!(original_content, content_after_run);
    assert!(content_after_run.contains("// This is a comment"));
    assert!(content_after_run.contains("// End of line comment"));
}

fn create_test_files(dir_path: &Path) {
    let file1_content = r#"// This is a comment
fn main() {
    let x = 5;
    println!("Hello");
}
"#;
    
    let file2_content = r#"// Another comment
struct Test {
    /// Doc comment should remain
    value: i32,
    name: String, // aicodeanalyzer: ignore
}
"#;
    
    let file3_content = r#"// This comment will be removed
fn test() {
    // aicodeanalyzer: ignore
    let y = 10;
}
"#;

    let file4_content = r#"
fn string_literals_with_comments() {
    let str1 = "This string contains // a comment-like pattern";
    let str2 = "Anthropic Claude";
    let str3 = "Multiple // comment // patterns";
    
    let raw_str = r##"This is a raw string with // comment pattern"##;
    
    let escaped_str = "Escaped quote \\\" and // comment";
    
    let formatted = format!("Generate code for task: {}", "task");
}
"#;
    
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
