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
        create_actionable_recommendations_prompt(code, batch_number, file_count)
    } else {
        create_full_analysis_prompt(code, batch_number, file_count)
    }
}

/// Creates the base prompt content shared by all clean code analysis prompts
fn create_shared_prompt_base() -> String {
    "Analyze the following code against these Clean Code principles:\n\
    - Use meaningful and intention-revealing names\n\
    - Functions should do one thing only and do it well\n\
    - Keep functions small (preferably under 30 lines)\n\
    - Arguments should be few (ideally 0-2, maximum 3 for non-configuration objects)\n\
    - Avoid side effects in functions\n\
    - Don't repeat yourself (DRY)\n\
    - Maintain clear separation of concerns\n\
    - Avoid unnecessary comments (code should be self-documenting)\n\n\
    IMPORTANT GUIDELINES:\n\
    - IGNORE all Rust documentation comments (triple slash '///'). These are API docs and are not violations.\n\
    - Only flag comments that explain 'what' instead of 'why' as unnecessary\n\
    - Consider Rust idioms and patterns as good practice, not violations\n\
    - Well-named utility functions are appropriate, even if they're small\n\
    - Configuration structs with many fields are acceptable for grouping related parameters".to_string()
}

/// Creates a prompt focused on high-value, actionable recommendations only
pub fn create_actionable_recommendations_prompt(code: String, batch_number: usize, file_count: usize) -> String {
    let base_prompt = create_shared_prompt_base();
    
    format!(
        "{}\n\n\
        ACTIONABLE RECOMMENDATIONS INSTRUCTIONS:\n\
        1. Focus ONLY on significant issues that would meaningfully improve the code\n\
        2. Provide a MAXIMUM of 3-5 recommendations total across all principles\n\
        3. Prioritize recommendations by impact (high, medium, low)\n\
        4. Include line numbers or function names in your recommendations\n\
        5. It is COMPLETELY ACCEPTABLE to report 'No significant issues found' if the code is well-structured\n\
        6. Do not manufacture issues or force recommendations when none are needed\n\
        7. For each recommendation, explain the specific benefit the change would bring\n\
        8. Include ONLY medium or high impact issues - ignore minor stylistic concerns\n\
        9. Consider the existing patterns in the codebase before suggesting changes\n\n\
        Recommendations format:\n\
        - If issues exist: List the 3-5 highest impact recommendations with clear rationale and benefit\n\
        - If no significant issues: Simply state 'No significant issues found. The code follows clean code principles well.'\n\n\
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
        1. For each principle, indicate whether the code follows it with specific examples\n\
        2. Be balanced in your analysis, covering both strengths and weaknesses\n\
        3. Provide actionable recommendations only for significant issues (medium or high impact)\n\
        4. Include line numbers or function names in your recommendations whenever possible\n\
        5. DO NOT force recommendations when they aren't needed - it's acceptable to praise good code\n\
        6. Limit recommendations to a maximum of 2-3 per principle\n\
        7. Consider the nature of the codebase and its patterns before suggesting changes\n\
        8. Be realistic about what constitutes a 'violation' vs. an acceptable trade-off\n\n\
        Analyze these {} files (Batch #{}):\n{}",
        base_prompt,
        file_count,
        batch_number,
        code
    )
}
