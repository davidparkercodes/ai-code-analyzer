/// Creates a complete AI prompt for clean code analysis based on file contents and settings
pub fn create_clean_code_prompt(
    file_contents: &[(String, String)],
    batch_number: usize,
    file_count: usize,
    actionable_only: bool,
    analyze_level: &str,
) -> String {
    let all_code = concatenate_file_contents(file_contents);
    
    create_prompt(actionable_only, all_code, batch_number, file_count, analyze_level)
}

/// Concatenates file paths and contents into a single string for the prompt
fn concatenate_file_contents(file_contents: &[(String, String)]) -> String {
    file_contents.iter()
        .map(|(path, content)| format!("\n\n// File: {}\n{}", path, content))
        .collect::<Vec<_>>()
        .join("")
}

/// Creates the appropriate type of prompt based on the actionable_only flag
fn create_prompt(actionable_only: bool, code: String, batch_number: usize, file_count: usize, analyze_level: &str) -> String {
    if actionable_only {
        create_actionable_recommendations_prompt(code, batch_number, file_count, analyze_level)
    } else {
        create_full_analysis_prompt(code, batch_number, file_count, analyze_level)
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
pub fn create_actionable_recommendations_prompt(code: String, batch_number: usize, file_count: usize, analyze_level: &str) -> String {
    let base_prompt = create_shared_prompt_base();
    let strictness_level = match analyze_level {
        "low" => {
            "MINIMAL STRICTNESS MODE INSTRUCTIONS:\n\
            1. ONLY flag the most critical violations of clean code principles\n\
            2. Provide a MAXIMUM of 1-2 recommendations, and ONLY if they represent significant issues\n\
            3. It is EXPECTED to report 'No significant issues found' for well-structured code\n\
            4. DO NOT suggest minor improvements - focus only on clear, objective violations\n\
            5. If the code follows clean code principles reasonably well, simply acknowledge it's good\n\n\
            For well-structured code, respond ONLY with:\n\
            'This code follows Clean Code principles well. No significant issues found.'\n"
        },
        "high" => {
            "COMPREHENSIVE STRICTNESS MODE INSTRUCTIONS:\n\
            1. Conduct a thorough, detailed analysis of all clean code principles\n\
            2. Provide up to 5-8 recommendations total across all principles\n\
            3. Include recommendations for minor improvements and stylistic concerns\n\
            4. Be specific and detailed in your analysis and recommendations\n\
            5. Consider both obvious violations and subtle optimization opportunities\n"
        },
        _ => { // Medium level (default)
            "STANDARD STRICTNESS MODE INSTRUCTIONS:\n\
            1. Focus ONLY on significant issues that would meaningfully improve the code\n\
            2. Provide a MAXIMUM of 3-5 recommendations total across all principles\n\
            3. Include ONLY medium or high impact issues - ignore minor stylistic concerns\n\
            4. It is ACCEPTABLE to report 'No significant issues found' if the code is well-structured\n\
            5. Do not manufacture issues or force recommendations when none are needed\n"
        }
    };
    
    format!(
        "{}\n\n\
        ACTIONABLE RECOMMENDATIONS INSTRUCTIONS:\n\
        {}\
        1. Prioritize recommendations by impact (high, medium, low)\n\
        2. Include line numbers or function names in your recommendations\n\
        3. For each recommendation, explain the specific benefit the change would bring\n\
        4. Consider the existing patterns in the codebase before suggesting changes\n\n\
        Recommendations format:\n\
        - If issues exist: List the highest impact recommendations with clear rationale and benefit\n\
        - If no significant issues: Simply state 'No significant issues found. The code follows clean code principles well.'\n\n\
        Analyze these {} files (Batch #{}):\n{}",
        base_prompt,
        strictness_level,
        file_count,
        batch_number,
        code
    )
}

/// Creates a prompt for comprehensive analysis of clean code principles
pub fn create_full_analysis_prompt(code: String, batch_number: usize, file_count: usize, analyze_level: &str) -> String {
    let base_prompt = create_shared_prompt_base();
    let strictness_level = match analyze_level {
        "low" => {
            "MINIMAL STRICTNESS MODE INSTRUCTIONS:\n\
            1. Focus primarily on highlighting the strengths of the code\n\
            2. Only mention the most significant clean code violations, if any\n\
            3. It is entirely appropriate to praise well-structured code without finding issues\n\
            4. Limit recommendations to only the most critical issues (1-2 at most)\n\
            5. For well-structured code, explicitly state that no significant issues were found\n"
        },
        "high" => {
            "COMPREHENSIVE STRICTNESS MODE INSTRUCTIONS:\n\
            1. Conduct a detailed analysis of all clean code principles\n\
            2. Consider even minor violations and stylistic improvements\n\
            3. Provide up to 4-5 recommendations per principle where appropriate\n\
            4. Look for subtle optimization opportunities and design pattern improvements\n\
            5. Be specific and detailed in your analysis of both strengths and weaknesses\n"
        },
        _ => { // Medium level (default)
            "STANDARD STRICTNESS MODE INSTRUCTIONS:\n\
            1. Be balanced in your analysis, covering both strengths and weaknesses\n\
            2. Provide actionable recommendations only for significant issues (medium or high impact)\n\
            3. Limit recommendations to a maximum of 2-3 per principle\n\
            4. DO NOT force recommendations when they aren't needed - it's acceptable to praise good code\n\
            5. Be realistic about what constitutes a 'violation' vs. an acceptable trade-off\n"
        }
    };
    
    format!(
        "{}\n\n\
        ANALYSIS INSTRUCTIONS:\n\
        {}\
        1. For each principle, indicate whether the code follows it with specific examples\n\
        2. Include line numbers or function names in your recommendations whenever possible\n\
        3. Consider the nature of the codebase and its patterns before suggesting changes\n\n\
        Analyze these {} files (Batch #{}):\n{}",
        base_prompt,
        strictness_level,
        file_count,
        batch_number,
        code
    )
}
