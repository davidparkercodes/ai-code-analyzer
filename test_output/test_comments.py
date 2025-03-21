# This is a test file for Python comment deletion testing
# It contains multiple single-line comments that should be deleted

### This is a Python doc comment that should be preserved
def some_function():
    a = 1  # This is an inline comment that should be removed
    b = 2
    
    # This entire line comment should be removed
    c = a + b  # Another inline comment

    '''
    This is a multi-line string (not a comment)
    and should be preserved
    '''
    
    # aicodeanalyzer: ignore - This comment should be preserved since it has the ignore marker
    d = c * 2  # This comment should be removed

    raw_string = r"This is a raw string with # inside not a comment"
    
    formatted = f"Value is {a}  # not a comment inside string"