use code_analyzer::style::detector::StyleDetector;
use code_analyzer::style::models::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_style_detector_basics() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a rust file with specific styling
    create_test_rust_file(&temp_path.join("test_file.rs"));
    
    // Run the detector
    let detector = StyleDetector::new();
    let analysis = detector.detect_styles(temp_path).unwrap();
    
    // Verify the analysis has content
    assert!(!analysis.file_profiles.is_empty());
    
    // Check that the consistency score is a valid value
    assert!(analysis.consistency_score >= 0.0 && analysis.consistency_score <= 1.0);
}

#[test]
fn test_style_detection_indentation() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create files with different indentation styles
    create_file_with_spaces(&temp_path.join("spaces.rs"), 4);
    create_file_with_tabs(&temp_path.join("tabs.rs"));
    
    // Run the detector
    let detector = StyleDetector::new();
    let analysis = detector.detect_styles(temp_path).unwrap();
    
    // Verify the analysis contains both files
    assert_eq!(analysis.file_profiles.len(), 2);
    
    // Verify the indentation was detected correctly
    let spaces_path = temp_path.join("spaces.rs").to_string_lossy().to_string();
    if let Some(spaces_profile) = analysis.file_profiles.get(&spaces_path) {
        match &spaces_profile.indentation {
            IndentationType::Spaces(n) => assert_eq!(*n, 4),
            _ => panic!("Expected Spaces(4) indentation, got {:?}", spaces_profile.indentation),
        }
    } else {
        panic!("spaces.rs profile not found");
    }
    
    let tabs_path = temp_path.join("tabs.rs").to_string_lossy().to_string();
    if let Some(tabs_profile) = analysis.file_profiles.get(&tabs_path) {
        match &tabs_profile.indentation {
            IndentationType::Tabs => (),
            _ => panic!("Expected Tabs indentation, got {:?}", tabs_profile.indentation),
        }
    } else {
        panic!("tabs.rs profile not found");
    }
    
    // Verify that indentation inconsistencies were detected
    assert!(!analysis.inconsistencies.is_empty());
}

fn create_test_rust_file(path: &Path) {
    let content = r#"
// This is a test Rust file with specific styling
use std::collections::HashMap;

fn main() {
    let snake_case_var = 42;
    let anotherVar = "camelCase";
    
    // A function with parameters
    process_data(snake_case_var, anotherVar);
    
    // Create a data structure
    let mut data = HashMap::new();
    data.insert("key1", "value1");
    data.insert("key2", "value2");
    
    println!("Done processing data: {:?}", data);
}

fn process_data(number: i32, text: &str) -> bool {
    if number > 40 {
        println!("Number is greater than 40: {}", number);
        println!("Text: {}", text);
        true
    } else {
        false
    }
}

struct TestStruct {
    field1: i32,
    field2: String,
}

impl TestStruct {
    fn new(f1: i32, f2: &str) -> Self {
        Self {
            field1: f1,
            field2: f2.to_string(),
        }
    }
    
    fn display(&self) {
        println!("TestStruct {{ field1: {}, field2: {} }}", 
                self.field1, self.field2);
    }
}
"#;

    fs::write(path, content).unwrap();
}

fn create_file_with_spaces(path: &Path, spaces: usize) {
    let indent = " ".repeat(spaces);
    let content = format!(r#"
fn main() {{
{0}println!("Hello, world!");
{0}let x = 42;
{0}if x > 40 {{
{0}{0}println!("Greater than 40");
{0}}}
}}
"#, indent);

    fs::write(path, content).unwrap();
}

fn create_file_with_tabs(path: &Path) {
    let content = r#"
fn main() {
	println!("Hello, world!");
	let x = 42;
	if x > 40 {
		println!("Greater than 40");
	}
}
"#;

    fs::write(path, content).unwrap();
}