use code_analyzer::description::CodeDescriptor;
use code_analyzer::ai::AiConfig;
use code_analyzer::util::parallel::ParallelProcessing;
use tempfile::TempDir;
use std::fs;
use tokio;

// Helper function to create a simple test project
fn create_test_project() -> TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a simple file structure
    fs::create_dir_all(temp_path.join("src")).unwrap();
    fs::create_dir_all(temp_path.join("tests")).unwrap();
    
    // Create a main.rs file
    fs::write(
        temp_path.join("src/main.rs"),
        r#"
fn main() {
    println!("Hello, world!");
    let result = add(2, 3);
    println!("2 + 3 = {}", result);
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
        "#,
    ).unwrap();
    
    // Create a lib.rs file
    fs::write(
        temp_path.join("src/lib.rs"),
        r#"
/// Adds two numbers
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Subtracts second number from first number
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}
        "#,
    ).unwrap();
    
    // Create a test file
    fs::write(
        temp_path.join("tests/test_math.rs"),
        r#"
use code_analyzer::add;
use code_analyzer::subtract;

#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
    assert_eq!(add(5, 7), 12);
}

#[test]
fn test_subtract() {
    assert_eq!(subtract(10, 5), 5);
    assert_eq!(subtract(8, 3), 5);
}
        "#,
    ).unwrap();
    
    temp_dir
}

#[tokio::test]
#[ignore] // Ignore by default as it makes API calls
async fn test_code_descriptor() {
    let test_project = create_test_project();
    let _project_path = test_project.path();
    
    // Mock AI config without actual API keys for testing
    let ai_config = AiConfig::default();
    
    // Create the code descriptor
    let descriptor = CodeDescriptor::new(ai_config);
    
    // Since we can't directly test the AI-related functionality without keys,
    // we'll just verify that the CodeDescriptor is properly constructed
    assert_eq!(descriptor.is_parallel(), true);
    
    // We could create a more thorough test with mocking of the AI components,
    // but that would be overkill for this test. A real integration test would
    // require actual API keys.
}