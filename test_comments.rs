// This comment will be removed
fn test_clean_comments() {
    let x = 5; // This comment will also be removed
    
    // This standalone comment line will be removed
    
    /// This documentation comment will be preserved
    let y = 10;
    
    // aicodeanalyzer: ignore
    let z = 15; // This comment will be removed
    
    if x > 3 { // This will be removed
        println!("x is greater than 3");
    }
}