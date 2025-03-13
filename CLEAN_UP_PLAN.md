# Code Cleanup Plan

This document outlines our strategy for cleaning up the codebase according to Clean Code principles and best practices. We'll organize cleanup by module/file to apply all relevant principles at once to each area.

## Clean Code Principles

- **Single Responsibility**: Functions should do one thing only and do it well
- **Descriptive Naming**: Names should reveal intent and be self-documenting
- **No Comments**: Code should be self-explanatory; create extra descriptive functions instead of comments
- **Small Functions**: Keep functions small (preferably under 20 lines)
- **Proper Organization**: Code should be organized in a logical way with clear separation of concerns

## Cleanup Status

### Completed Work ✅
- ✅ Split `markdown.rs` into: `markdown/formatter.rs`, `markdown/renderer.rs`, `markdown/syntax_highlighter.rs`
- ✅ Extract command handling from main.rs into separate command modules
- ✅ Create shared utility for parallel processing flags
- ✅ Remove unnecessary comments from key files (Phase 1)

### Pending Work - By Module

### Commands Module
- **`src/commands/dependencies.rs`**:
  - Refactor `execute()` function into smaller single-responsibility functions
  - Fix error variable naming (`e` → `error`)
  - Remove duplicated error handling
  - Improve function organization

- **`src/commands/metrics.rs`**:
  - Refactor `execute()` function
  - Fix error variable naming
  - Improve parallel processing naming and organization

- **`src/commands/run.rs`**:
  - Refactor to reduce complexity
  - Streamline error handling
  - Fix variable naming

### Analyzer Module
- **`src/analyzer/file_analyzer.rs`**:
  - Break down `analyze_file` method (75+ lines) into:
    - `detect_file_language`
    - `count_metrics`
    - `check_test_file`
  - Fix any naming issues 
  - Remove any remaining unnecessary comments

### Metrics Module
- **`src/metrics/collector.rs`**:
  - Split `collect_metrics` into:
    - `walk_directory`
    - `filter_entries`
    - `process_entries`
  - Rename metrics variables (`loc` → `lines_of_code`, etc.)

- **`src/metrics/models.rs`**:
  - Improve variable naming
  - Streamline any complex methods

### Output Module
- **`src/output/markdown/formatter.rs`**:
  - Refactor `format_markdown` into multiple focused functions for each markdown element
  - Improve naming throughout

- **`src/output/markdown/syntax_highlighter.rs`**:
  - Refactor `highlight_rust` to reduce complexity
  - Break down into smaller functions

### Error Handling Improvements (All Modules)
- Create consistent error handling strategy
- Replace String errors with proper error enums
- Extract command error handling into helper functions

### Parallel Processing (All Modules)
- Rename `with_parallel` to `enable_parallel_processing`
- Ensure consistent parallel processing patterns

## Workflow Instructions

For each module cleanup:
1. Create a separate commit for each logical file/module
2. Apply all clean code principles to that file/module:
   - Break down large functions
   - Fix naming
   - Remove unnecessary comments
   - Improve error handling
3. Update this document to mark completed items
4. Document lessons learned and improvements made

Each commit should be focused on a specific module/file improvement to maintain a clear history of changes.