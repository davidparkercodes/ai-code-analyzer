
/// Creates a complete AI prompt for clean code analysis with JSON output format
pub fn create_clean_code_json_prompt(
    file_contents: &[(String, String)],
    batch_number: usize,
    file_count: usize,
    actionable_only: bool,
    analyze_level: &str,
) -> String {
    let all_code = concatenate_file_contents(file_contents);
    
    create_json_prompt(actionable_only, all_code, batch_number, file_count, analyze_level)
}

/// Concatenates file paths and contents into a single string for the prompt
fn concatenate_file_contents(file_contents: &[(String, String)]) -> String {
    file_contents.iter()
        .map(|(path, content)| format!("\n\n// File: {}\n{}", path, content))
        .collect::<Vec<_>>()
        .join("")
}


/// Creates the appropriate type of JSON prompt based on the actionable_only flag
fn create_json_prompt(actionable_only: bool, code: String, batch_number: usize, file_count: usize, analyze_level: &str) -> String {
    if actionable_only {
        create_actionable_json_prompt(code, batch_number, file_count, analyze_level)
    } else {
        create_full_json_prompt(code, batch_number, file_count, analyze_level)
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

/// Instructions for scoring based on analyze level
fn get_scoring_instructions(analyze_level: &str) -> String {
    match analyze_level {
        "low" => "Be generous with your scoring - assign higher scores (85-100) for code that follows most principles".to_string(),
        "high" => "Be strict with your scoring - even well-structured code should not receive a perfect score\n\
                  The score should reflect that there's always room for improvement".to_string(),
        _ => "Score moderately - well-structured code should receive scores in the 75-90 range\n\
              Reserve scores above 90 for exceptional code with minimal issues".to_string()
    }
}


/// Get strictness level instructions for actionable recommendations
fn get_actionable_strictness_instructions(analyze_level: &str) -> String {
    match analyze_level {
        "low" => {
            "MINIMAL STRICTNESS MODE INSTRUCTIONS:\n\
            1. ONLY flag the most critical violations of clean code principles\n\
            2. Provide a MAXIMUM of 1-2 recommendations, and ONLY if they represent significant issues\n\
            3. It is EXPECTED to report 'No significant issues found' for well-structured code\n\
            4. DO NOT suggest minor improvements - focus only on clear, objective violations\n\
            5. If the code follows clean code principles reasonably well, simply acknowledge it's good\n\n\
            For well-structured code, start with your score followed by:\n\
            'No significant issues found. The code follows clean code principles well.'"
        },
        "high" => {
            "COMPREHENSIVE STRICTNESS MODE INSTRUCTIONS:\n\
            1. Conduct a thorough, detailed analysis of all clean code principles\n\
            2. Provide up to 5-8 recommendations total across all principles\n\
            3. Include recommendations for minor improvements and stylistic concerns\n\
            4. Be specific and detailed in your analysis and recommendations\n\
            5. Consider both obvious violations and subtle optimization opportunities"
        },
        _ => {
            "STANDARD STRICTNESS MODE INSTRUCTIONS:\n\
            1. Focus ONLY on significant issues that would meaningfully improve the code\n\
            2. Provide a MAXIMUM of 3-5 recommendations total across all principles\n\
            3. Include ONLY medium or high impact issues - ignore minor stylistic concerns\n\
            4. It is ACCEPTABLE to report 'No significant issues found' if the code is well-structured\n\
            5. Do not manufacture issues or force recommendations when none are needed"
        }
    }.to_string()
}




/// Get strictness level instructions for full analysis
fn get_analysis_strictness_instructions(analyze_level: &str) -> String {
    match analyze_level {
        "low" => {
            "MINIMAL STRICTNESS MODE INSTRUCTIONS:\n\
            1. Conduct a fair, balanced review without bias toward strengths or weaknesses\n\
            2. Only mention the most significant clean code violations, if any\n\
            3. It is entirely appropriate to note that well-structured code has no significant issues\n\
            4. Limit recommendations to only the most critical issues (1-2 at most)\n\
            5. For well-structured code, explicitly state that no significant issues were found"
        },
        "high" => {
            "COMPREHENSIVE STRICTNESS MODE INSTRUCTIONS:\n\
            1. Conduct a detailed analysis of all clean code principles\n\
            2. Consider even minor violations and stylistic improvements\n\
            3. Provide up to 4-5 recommendations per principle where appropriate\n\
            4. Look for subtle optimization opportunities and design pattern improvements\n\
            5. Be specific and detailed in your analysis of both strengths and weaknesses"
        },
        _ => {
            "STANDARD STRICTNESS MODE INSTRUCTIONS:\n\
            1. Be balanced in your analysis, covering both strengths and weaknesses\n\
            2. Provide actionable recommendations only for significant issues (medium or high impact)\n\
            3. Limit recommendations to a maximum of 2-3 per principle\n\
            4. DO NOT force recommendations when they aren't needed - it's acceptable to praise good code\n\
            5. Be realistic about what constitutes a 'violation' vs. an acceptable trade-off"
        }
    }.to_string()
}




/// Get JSON format instructions for output
fn get_json_output_format() -> String {
    "REQUIRED JSON OUTPUT FORMAT:\n\
    You MUST provide your analysis results as a valid JSON array of objects, where each object has the following structure in this EXACT order:\n\
    {\n\
      \"file\": \"filename.rs\",     // The filename from the code blocks\n\
      \"score\": 85,               // A number from 0-100 representing the clean code score\n\
      \"actionableItems\": [       // Array of objects containing structured recommendations\n\
        {\n\
          \"location\": \"function_name()\",  // The function, method, or code section where the issue is found\n\
          \"lineNumber\": 42,                // The approximate line number where the issue occurs\n\
          \"recommendation\": \"Update function to follow single responsibility principle by splitting into two functions\"  // The actual recommendation\n\
        }\n\
      ]\n\
    }\n\
    \n\
    IMPORTANT: The properties MUST appear IN THIS ORDER in the JSON output:\n\
    1. file\n\
    2. score\n\
    3. actionableItems\n\
    \n\
    CRITICAL REQUIREMENTS:\n\
    - Your response must ONLY contain a valid JSON array, with NO text before or after\n\
    - Do not include any explanations, introductions, or markdown formatting\n\
    - For files with no issues, include an empty array for actionableItems\n\
    - Every recommendation MUST include location, lineNumber, and recommendation fields\n\
    - If you can't determine the exact line number, provide your best estimate\n\
    - Each recommendation should explain both WHAT to change and WHY it improves the code\n\
    - Follow the strictness level when determining what issues to include\n\
    - All recommendations should be concise but clear, focused on tangible improvements\n\
    - Use camelCase for property names (actionableItems, lineNumber)".to_string()
}

/// Creates a prompt for actionable recommendations in JSON format
pub fn create_actionable_json_prompt(code: String, batch_number: usize, file_count: usize, analyze_level: &str) -> String {
    let base_prompt = create_shared_prompt_base();
    let strictness_instructions = get_actionable_strictness_instructions(analyze_level);
    let scoring_instructions = get_scoring_instructions(analyze_level);
    let json_format = get_json_output_format();
    
    format!(
        "{}\n\n\
        ACTIONABLE RECOMMENDATIONS INSTRUCTIONS:\n\
        {}\n\
        {}.\n\n\
        {}\n\n\
        Remember: Your output must be ONLY valid JSON with no additional text.\n\n\
        Analyze these {} files (Batch #{}):\n{}",
        base_prompt,
        strictness_instructions,
        scoring_instructions,
        json_format,
        file_count,
        batch_number,
        code
    )
}

/// Creates a prompt for full analysis in JSON format
pub fn create_full_json_prompt(code: String, batch_number: usize, file_count: usize, analyze_level: &str) -> String {
    let base_prompt = create_shared_prompt_base();
    let strictness_instructions = get_analysis_strictness_instructions(analyze_level);
    let scoring_instructions = get_scoring_instructions(analyze_level);
    let json_format = get_json_output_format();
    
    format!(
        "{}\n\n\
        ANALYSIS INSTRUCTIONS:\n\
        {}\n\
        {}.\n\n\
        {}\n\n\
        Remember: Your output must be ONLY valid JSON with no additional text.\n\n\
        Analyze these {} files (Batch #{}):\n{}",
        base_prompt,
        strictness_instructions,
        scoring_instructions,
        json_format,
        file_count,
        batch_number,
        code
    )
}
