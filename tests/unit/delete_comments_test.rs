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
        "rust".to_string(),
        false,
        Some(output_path.clone()),
        true,
        true,
        true,
        true
    );
    
    assert_eq!(exit_code, 0);
    
    let file1_path = temp_path.join("test1.rs");
    let file2_path = temp_path.join("test2.rs");
    let file3_path = temp_path.join("test3.rs");
    let file4_path = temp_path.join("test4.rs");
    
    let original_file1 = fs::read_to_string(&file1_path).unwrap();
    assert!(original_file1.contains("// This is a comment"));
    assert!(original_file1.contains("// End of line comment"));
    
    if let Ok(entries) = fs::read_dir(&output_path) {
        println!("Files in output directory:");
        for entry in entries.filter_map(|e| e.ok()) {
            println!("  {}", entry.path().display());
        }
    }
}

#[test]
fn test_dry_run_mode() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    create_test_files(temp_path);
    
    let file_extension = get_rs_extension();
    let filename1 = format!("test1{}", file_extension);
    let test_file_path = temp_path.join(&filename1);
    
    let original_content = fs::read_to_string(&test_file_path).unwrap();
    
    let exit_code = delete_comments::execute(
        test_file_path.to_str().unwrap().to_string(),
        "rust".to_string(),
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

#[test]
fn test_delete_comments_from_python_files() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    create_python_test_files(temp_path);
    
    let output_dir = TempDir::new().unwrap();
    let output_path = output_dir.path().to_str().unwrap().to_string();
    
    let exit_code = delete_comments::execute(
        temp_path.to_str().unwrap().to_string(),
        "python".to_string(),
        false,
        Some(output_path.clone()),
        true,
        true,
        true,
        true
    );
    
    assert_eq!(exit_code, 0);
    
    let file1_path = temp_path.join("test1.py");
    let file2_path = temp_path.join("test2.py");
    
    let original_file1 = fs::read_to_string(&file1_path).unwrap();
    assert!(original_file1.contains("# This is a comment"));
    assert!(original_file1.contains("# End of line comment"));
    
    if let Ok(entries) = fs::read_dir(&output_path) {
        println!("Python files in output directory:");
        for entry in entries.filter_map(|e| e.ok()) {
            println!("  {}", entry.path().display());
        }
    }
}

fn get_rs_extension() -> String {
    format!("{}{}", ".", "rs")
}

fn create_test_files(dir_path: &Path) {
    let file1_content = r###"// This is a comment
fn main() {
    let x = 5;
    println!("Hello");
}
"###;
    
    let file2_content = r###"// Another comment
struct Test {
    /// Doc comment should remain
    value: i32,
    name: String, // aicodeanalyzer: ignore
}
"###;
    
    let file3_content = r###"// This comment will be removed
fn test() {
    // aicodeanalyzer: ignore
    let y = 10;
}
"###;

    let file4_content = r###"
fn string_literals_with_comments() {
    let str1 = "This string contains // a comment-like pattern";
    let str2 = "Anthropic Claude";
    let str3 = "Multiple // comment // patterns";
    
    let raw_str = r##"This is a raw string with // comment pattern"##;
    
    let escaped_str = "Escaped quote \\\" and // comment";
    
    let formatted = format!("Generate code for task: {}", "task");
}
"###;
    
    let file_extension = get_rs_extension();
    
    let filename1 = format!("test1{}", file_extension);
    let filename2 = format!("test2{}", file_extension);
    let filename3 = format!("test3{}", file_extension);
    let filename4 = format!("test4{}", file_extension);
    
    let test1_path = dir_path.join(&filename1);
    let test2_path = dir_path.join(&filename2);
    let test3_path = dir_path.join(&filename3);
    let test4_path = dir_path.join(&filename4);
    
    fs::write(&test1_path, file1_content).unwrap();
    fs::write(&test2_path, file2_content).unwrap();
    fs::write(&test3_path, file3_content).unwrap();
    fs::write(&test4_path, file4_content).unwrap();
}

fn create_python_test_files(dir_path: &Path) {
    let file1_content = r###"# This is a comment
def main():
    x = 5  # End of line comment
    print("Hello")
"###;
    
    let file2_content = r###"# Another comment
class Test:
    ### Doc comment should remain
    def __init__(self, value):
        self.value = value
        self.name = "Test"  # aicodeanalyzer: ignore
"###;
    
    let file3_content = r###"# This comment will be removed
def test():
    # aicodeanalyzer: ignore
    y = 10
"###;

    let file4_content = r###"
def string_literals_with_comments():
    str1 = "This string contains # a comment-like pattern"
    str2 = "Anthropic Claude"
    str3 = "Multiple # comment # patterns"
    
    raw_str = '''This is a multi-line string with # comment pattern'''
    
    escaped_str = "Escaped quote \\\" and # comment"
    
    formatted = f"Generate code for task: {task}"
"###;
    
    let filename1 = "test1.py";
    let filename2 = "test2.py";
    let filename3 = "test3.py";
    let filename4 = "test4.py";
    
    let test1_path = dir_path.join(&filename1);
    let test2_path = dir_path.join(&filename2);
    let test3_path = dir_path.join(&filename3);
    let test4_path = dir_path.join(&filename4);
    
    fs::write(&test1_path, file1_content).unwrap();
    fs::write(&test2_path, file2_content).unwrap();
    fs::write(&test3_path, file3_content).unwrap();
    fs::write(&test4_path, file4_content).unwrap();
}
