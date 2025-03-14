fn test_clean_comments() {
    let x = 5; 
    
    
    /// This documentation comment will be preserved
    let y = 10;
    
    // aicodeanalyzer: ignore
    let z = 15; 
    
    if x > 3 { 
        println!("x is greater than 3");
    }
}
