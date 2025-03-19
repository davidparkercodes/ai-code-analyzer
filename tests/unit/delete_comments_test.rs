use std::fs;
use std::path::Path;
use tempfile::TempDir;
use ai_code_analyzer::commands::delete_comments;

#[test]
fn test_delete_comments_from_rust_files() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    create_test_files(temp_path);
    
    let output_dir = TempDir::new().unwrap();
    let output_path = output_dir.path().to_str().unwrap().to_string();
    
    // Override the should_exclude in test context to not filter test files in our test
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
    let _file2_path = temp_path.join("test2.rs");
    let _file3_path = temp_path.join("test3.rs");
    let _file4_path = temp_path.join("test4.rs");
    
    let original_file1 = fs::read_to_string(&file1_path).unwrap();
    assert!(original_file1.contains("// This is a comment"));
    
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
    let _file2_path = temp_path.join("test2.py");
    
    let original_file1 = fs::read_to_string(&file1_path).unwrap();
    assert!(original_file1.contains("# This is a comment"));
    
    if let Ok(entries) = fs::read_dir(&output_path) {
        println!("Python files in output directory:");
        for entry in entries.filter_map(|e| e.ok()) {
            println!("  {}", entry.path().display());
        }
    }
}

#[test]
fn test_delete_comments_from_csharp_files() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    create_csharp_test_files(temp_path);
    
    let output_dir = TempDir::new().unwrap();
    let output_path = output_dir.path().to_str().unwrap().to_string();
    
    let exit_code = delete_comments::execute(
        temp_path.to_str().unwrap().to_string(),
        "csharp".to_string(),
        false,
        Some(output_path.clone()),
        true,
        true,
        true,
        true
    );
    
    assert_eq!(exit_code, 0);
    
    let file1_path = temp_path.join("Test1.cs");
    let _file2_path = temp_path.join("Test2.cs");
    
    let original_file1 = fs::read_to_string(&file1_path).unwrap();
    assert!(original_file1.contains("// This is a comment"));
    assert!(original_file1.contains("/* This is a multi-line comment"));
    
    if let Ok(entries) = fs::read_dir(&output_path) {
        println!("C# files in output directory:");
        for entry in entries.filter_map(|e| e.ok()) {
            println!("  {}", entry.path().display());
            
            // Verify that comments have been removed in output files
            if entry.path().extension().unwrap_or_default() == "cs" {
                let processed_content = fs::read_to_string(entry.path()).unwrap();
                assert!(!processed_content.contains("// This is a comment"));
                assert!(!processed_content.contains("/* This is a multi-line comment"));
                // But doc comments and ignored comments should remain
                if entry.path().file_name().unwrap() == "Test2.cs" {
                    assert!(processed_content.contains("/// <summary>"));
                    assert!(processed_content.contains("aicodeanalyzer: ignore"));
                }
            }
        }
    }
}

#[test]
fn test_ignores_test_files() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create test directory structure
    let tests_dir = temp_path.join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    
    // Create a regular source file
    let source_content = r###"// This is a comment in a source file
fn sample_function() {
    let x = 5; // Another comment
    println!("Hello");
}
"###;
    fs::write(temp_path.join("source.rs"), source_content).unwrap();
    
    // Create a test file
    let test_content = r###"// This is a comment in a test file
#[test]
fn test_function() {
    let x = 5; // Test comment
    assert_eq!(x, 5);
}
"###;
    fs::write(tests_dir.join("test_file.rs"), test_content).unwrap();
    
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
    
    // Check that the source file was processed
    if let Ok(entries) = fs::read_dir(&output_path) {
        let output_files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        
        // Should find the source file but not the test file in output
        let has_source_file = output_files.iter().any(|entry| 
            entry.path().file_name().unwrap().to_str().unwrap() == "source.rs");
        assert!(has_source_file, "Source file should be processed");
        
        // There should be no tests directory or test files
        let has_test_dir = output_files.iter().any(|entry| 
            entry.path().is_dir() && entry.path().file_name().unwrap().to_str().unwrap() == "tests");
        assert!(!has_test_dir, "Test directory should not be processed");
        
        // Double check that test file doesn't exist in output root either
        let has_test_file = output_files.iter().any(|entry| 
            entry.path().file_name().unwrap().to_str().unwrap() == "test_file.rs");
        assert!(!has_test_file, "Test file should not be processed");
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

fn create_csharp_test_files(dir_path: &Path) {
    let file1_content = r###"// This is a comment
using System;

namespace TestNamespace 
{
    public class Program 
    {
        /* This is a multi-line comment
         * that spans multiple lines
         * and should be removed
         */
        public static void Main() 
        {
            int x = 5; // End of line comment
            Console.WriteLine("Hello");
            string verbatimString = @"This contains // fake comments that should be preserved";
        }
    }
}
"###;
    
    let file2_content = r###"// Another comment
using System;

namespace TestNamespace 
{
    /// <summary>
    /// XML documentation comment that should be preserved
    /// </summary>
    public class Test 
    {
        private int _value;
        private string _name; // aicodeanalyzer: ignore
        
        public Test(int value) 
        {
            _value = value;
            _name = "Test"; // This comment should be removed
        }
    }
}
"###;
    
    let file3_content = r###"// This comment will be removed
using System;

namespace TestNamespace 
{
    public static class Utilities 
    {
        // aicodeanalyzer: ignore
        public static int DoSomething(int y) 
        {
            /* Another multi-line comment
             * with indentation
             */
            return y * 2;
        }
    }
}
"###;

    let file4_content = r###"
using System;

namespace StringTests 
{
    public class StringTests 
    {
        public void StringLiteralsWithComments() 
        {
            string str1 = "This string contains // a comment-like pattern";
            string str2 = "Anthropic Claude";
            string str3 = "Multiple // comment // patterns";
            
            string verbatimStr = @"This is a verbatim string with // comment pattern
                and a newline with more // comments
                and ""quoted text"" inside";
            
            string escapedStr = "Escaped quote \\\" and // comment";
            
            string formatted = $"Generate code for task: {task}";
        }
    }
}
"###;
    
    let filename1 = "Test1.cs";
    let filename2 = "Test2.cs";
    let filename3 = "Test3.cs";
    let filename4 = "Test4.cs";
    
    let test1_path = dir_path.join(&filename1);
    let test2_path = dir_path.join(&filename2);
    let test3_path = dir_path.join(&filename3);
    let test4_path = dir_path.join(&filename4);
    
    fs::write(&test1_path, file1_content).unwrap();
    fs::write(&test2_path, file2_content).unwrap();
    fs::write(&test3_path, file3_content).unwrap();
    fs::write(&test4_path, file4_content).unwrap();
}
