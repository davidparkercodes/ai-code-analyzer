/// Creates a complete AI prompt for clean code analysis based on file contents and settings
pub fn create_clean_code_prompt(
    file_contents: &[(String, String)],
    batch_number: usize,
    file_count: usize,
    only_recommendations: bool,
) -> String {
    let all_code = concatenate_file_contents(file_contents);
    
    create_prompt(only_recommendations, all_code, batch_number, file_count)
}

/// Concatenates file paths and contents into a single string for the prompt
fn concatenate_file_contents(file_contents: &[(String, String)]) -> String {
    file_contents.iter()
        .map(|(path, content)| format!("\n\n// File: {}\n{}", path, content))
        .collect::<Vec<_>>()
        .join("")
}

/// Creates the appropriate type of prompt based on the only_recommendations flag
fn create_prompt(only_recommendations: bool, code: String, batch_number: usize, file_count: usize) -> String {
    if only_recommendations {
        create_recommendations_prompt(code, batch_number, file_count)
    } else {
        create_full_analysis_prompt(code, batch_number, file_count)
    }
}

/// Creates the base prompt content shared by all clean code analysis prompts
fn create_shared_prompt_base() -> String {
    "Analyze the following code against these Clean Code principles:\n\
    - Use meaningful and intention-revealing names\n\
    - Functions should do one thing only and do it well\n\
    - Keep functions small (preferably under 20 lines)\n\
    - Arguments should be few (ideally 0-2, maximum 3)\n\
    - Avoid side effects in functions\n\
    - Don't repeat yourself (DRY)\n\
    - Maintain clear separation of concerns\n\
    - Avoid unnecessary comments (code should be self-documenting)\n\n\
    Regarding comments specifically:\n\
    - Consider ALL inline comments that describe what the code is doing as unnecessary\n\
    - Comments like '// Initialize variables', '// Create batches', '// Process each batch' are violations\n\
    - Comments that explain 'what' instead of 'why' are unnecessary\n\
    - Even simple section divider comments should be counted as violations\n\
    - Good code should not need explanatory comments - the code itself should be clear enough".to_string()
}

/// Creates a prompt focused only on recommendations and violations
pub fn create_recommendations_prompt(code: String, batch_number: usize, file_count: usize) -> String {
    let base_prompt = create_shared_prompt_base();
    
    format!(
        "{}\n\n\
        CRITICAL INSTRUCTIONS - READ CAREFULLY:\n\
        1. You are in VIOLATIONS-ONLY MODE. Do NOT provide positive feedback or praise.\n\
        2. For each principle, ONLY identify violations and problematic code - nothing else.\n\
        3. DO NOT start sections with phrases like \"Generally Good\" or add checkmarks.\n\
        4. DO NOT mention good practices or compliments about the code - focus exclusively on issues.\n\
        5. NEVER write things like \"✅ Good\" or \"Excellent\" - ONLY report problems.\n\
        6. DO conduct a THOROUGH check for ALL comments in the code and flag them as violations.\n\
        7. Format your response as a list of violations and actionable recommendations ONLY.\n\
        8. If there are no violations for a principle, simply state \"No violations found\" and move on.\n\n\
        Analyze these {} files (Batch #{}):\n{}",
        base_prompt,
        file_count,
        batch_number,
        code
    )
}

/// Creates a prompt for comprehensive analysis of clean code principles
pub fn create_full_analysis_prompt(code: String, batch_number: usize, file_count: usize) -> String {
    let base_prompt = create_shared_prompt_base();
    
    format!(
        "{}\n\n\
        ANALYSIS INSTRUCTIONS:\n\
        1. For each principle, indicate whether the code follows it with specific examples.\n\
        2. Be thorough and detailed in your analysis, covering both strengths and weaknesses.\n\
        3. Provide actionable recommendations for areas that need improvement.\n\
        4. Be particularly thorough in checking for unnecessary comments in the code.\n\
        5. Include line numbers or function names in your recommendations whenever possible.\n\
        6. If code follows good practices in an area, acknowledge this but don't force recommendations.\n\
        7. Be honest in your assessment - don't mark something as good if it has clear issues.\n\
        8. Pay special attention to examining ALL comments in the code and evaluate if they are truly necessary.\n\n\
        Analyze these {} files (Batch #{}):\n{}",
        base_prompt,
        file_count,
        batch_number,
        code
    )
}
