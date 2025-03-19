// This is a test file for C# comment deletion testing
// It contains multiple single-line comments that should be deleted

/// <summary>
/// This is a C# XML doc comment that should be preserved
/// </summary>
public void SomeFunction()
{
    int a = 1; // This is an inline comment that should be removed
    int b = 2;
    
    // This entire line comment should be removed
    int c = a + b; // Another inline comment

    /* This is a multi-line comment
     * that spans multiple lines
     * and should also be properly handled
     */
    
    string verbatimString = @"This is a verbatim string with ""quotes"" inside // not a comment";
}

// aicodeanalyzer: ignore - This comment should be preserved since it has the ignore marker
public void IgnoredFunction()
{
    // This comment should be removed
}