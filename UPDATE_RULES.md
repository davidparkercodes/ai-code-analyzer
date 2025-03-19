# Claude's Memory File

## Project Overview

AI CodeAnalyzer is a high-performance, Rust-powered CLI tool designed to analyze codebases. It extracts metrics, detects coding styles, generates AI-ready summaries, maps architecture and dependencies, and more. Built with Rust for memory safety, performance, concurrency, and reliability, it targets developers, AI engineers, DevOps teams, and software architects.

## Comments Policy

-   ⚠️ NO UNNECESSARY COMMENTS IN CODE - code should be self-documenting
-   ⚠️ NO UNNECESSARY COMMENTS IN CODE - code should be self-documenting
-   ⚠️ NO UNNECESSARY COMMENTS IN CODE -- when you make changes to the code, DO NOT ADD UNNECESSARY COMMENTS - code should be self-documenting
-   ⚠️ NEVER add comments to implementation files - code should be self-documenting
-   ⚠️ ONLY add explanatory comments to test files for describing test setup and assertions
-   ⚠️ No TODOs, no explanatory comments, no commented-out code in implementation files
-   ⚠️ Use meaningful variable and function names instead of comments

## CRITICAL: NO AI ATTRIBUTION

-   ⚠️ NEVER include "Generated with Claude Code", "Co-Authored-By: Claude", or ANY similar attribution in:
    -   Git commits
    -   Pull requests
    -   Code or comments
    -   Commit messages
    -   PR descriptions
    -   Documentation
    -   Any other files or outputs
-   ⚠️ NEVER mention Claude, AI, or any form of AI assistance anywhere in the codebase or commit history
-   ⚠️ When making git commits, NEVER add any co-author or attribution lines

## AI Integration

-   Provider-specific model mapping:
    -   **Low tier**: Smaller, faster, cost-effective models for simple tasks
    -   **Medium tier**: Balanced models for regular analysis tasks
    -   **High tier**: Powerful models for complex code understanding

## Code Quality Standards

-   Follow Clean Code principles:
    -   Use meaningful and intention-revealing names
    -   Functions should do one thing only and do it well
    -   Keep functions small (preferably under 20 lines)
    -   Arguments should be few (ideally 0-2, maximum 3)
    -   Avoid side effects in functions
    -   Don't repeat yourself (DRY)
    -   Maintain clear separation of concerns
-   Follow existing Rust formatting conventions
-   Use idiomatic Rust patterns
-   Always add or update unit tests when adding or modifying functionality
-   Aim for high test coverage, especially for complex logic
-   Each public function/method should have corresponding test cases
-   Prefer composition over inheritance
-   Implement Default trait for structs with new() methods
-   Follow Rust's ownership model correctly
-   Use error handling consistently (Result/Option)

## Git Workflow

1. Always work in feature branches:
    ```
    git checkout main
    git pull
    git checkout -b feature/descriptive-name
    ```
2. Make focused, atomic commits with clear messages:
    ```
    git add .
    git commit -m "Implement feature X"
    ```
3. Push to remote and create PR:
    ```
    git push origin feature/descriptive-name
    ```
4. Open GitHub to create the PR, providing:
    - Clear title describing the change
    - Detailed description with context and motivation
    - List of changes and affected components
    - Testing information

## Important Notes

-   Maintain the modular architecture to allow for future extension
-   All public APIs should have clear interfaces and documentation
-   Handle errors gracefully with informative messages

## Recent Updates

### Improved delete-comments command git integration (2025-03-19)

The delete-comments command now properly detects whether a git repository has a remote origin configured.

**Changes made:**
1. Added detection of origin remote before attempting to push changes
2. Added clear messaging when skipping push and PR creation
3. Fixed error handling for git push failures
4. Updated the initial operation description to reflect the actual steps based on remote configuration

This improvement ensures the delete-comments command works correctly in local-only git repositories without attempting to push or create PRs when there's no remote.

### Added C#/.NET support to delete-comments command (2025-03-18)

The delete-comments command now supports C#/.NET projects for comment removal.

**Changes made:**
1. Added C# language option to the delete-comments command
2. Added proper handling for C# code comments, including XML doc comments
3. Enhanced string handling to support C# verbatim strings
4. Added C#-specific test file patterns for proper test file exclusion
5. Created C# test file to verify functionality

This enhancement allows the delete-comments command to be used with C#/.NET codebases, making the tool more versatile.

### Fixed delete-comments command to properly ignore test files (2025-03-16)

The delete-comments command now properly ignores test files during processing. 

**Changes made:**
1. Updated `should_exclude()` method in `FileFilter` to include `is_test_file()` in the exclusion check
2. Added a test case to verify test files are properly ignored
3. Fixed existing tests that were affected by this change

This ensures that when running the delete-comments command, it will not modify any test files, preserving test cases.
