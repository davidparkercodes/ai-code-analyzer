use code_analyzer::description::CodeDescriptor;
use code_analyzer::ai::AiConfig;
use code_analyzer::util::parallel::ParallelProcessing;
use tempfile::TempDir;
use std::fs;
use tokio;

fn create_test_project() -> TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();
    
    fs::create_dir_all(temp_path.join("src")).unwrap();
    fs::create_dir_all(temp_path.join("tests")).unwrap();
    
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
#[ignore]
async fn test_code_descriptor() {
    let test_project = create_test_project();
    let _project_path = test_project.path();
    
    let ai_config = AiConfig::default();
    
    let descriptor = CodeDescriptor::new(ai_config);
    
    assert_eq!(descriptor.is_parallel(), true);
    
}
