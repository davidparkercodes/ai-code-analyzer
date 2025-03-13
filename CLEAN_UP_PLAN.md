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

### Commands Module ✅
- ✅ **`src/commands/dependencies.rs`**:
  - ✅ Refactor `execute()` function into smaller single-responsibility functions
  - ✅ Fix error variable naming (`e` → `error`)
  - ✅ Remove duplicated error handling
  - ✅ Improve function organization

- ✅ **`src/commands/metrics.rs`**:
  - ✅ Refactor `execute()` function
  - ✅ Fix error variable naming
  - ✅ Improve parallel processing naming and organization

- ✅ **`src/commands/run.rs`**:
  - ✅ Refactor to reduce complexity
  - ✅ Streamline error handling
  - ✅ Fix variable naming

- ✅ **`src/commands/describe.rs`**:
  - ✅ Refactor `execute()` function into smaller single-responsibility functions
  - ✅ Fix error variable naming 
  - ✅ Improve function organization

- ✅ **`src/commands/style.rs`**:
  - ✅ Refactor `execute()` function into smaller functions
  - ✅ Fix error variable naming
  - ✅ Improve organization and error handling

- ✅ **`src/commands/mod.rs` & `src/main.rs`**:
  - ✅ Update to handle error return codes properly
  - ✅ Improve error propagation

### Analyzer Module ✅
- ✅ **`src/analyzer/file_analyzer.rs`**:
  - ✅ Break down `analyze_file` method (75+ lines) into:
    - ✅ `get_cached_metrics`
    - ✅ `detect_file_language`
    - ✅ `get_file_content`
    - ✅ `check_if_test_file`
    - ✅ `cache_file_metrics`
  - ✅ Fix naming issues (loc → lines_of_code, blank → blank_lines, comments → comment_lines)
  - ✅ Removed unnecessary comments

- ✅ **`src/analyzer/mod.rs`**:
  - ✅ Break down `analyze` method into:
    - ✅ `print_analysis_header`
    - ✅ `perform_metrics_analysis`
    - ✅ `perform_dependency_analysis`
    - ✅ `print_total_analysis_time`
  - ✅ Fix error variable naming (e → error)

### Metrics Module ✅
- ✅ **`src/metrics/collector.rs`**:
  - ✅ Split `collect_metrics` into:
    - ✅ `validate_directory_path`
    - ✅ `walk_directory`
    - ✅ `count_directories`
    - ✅ `filter_file_entries`
    - ✅ `process_file_entries`
    - ✅ `finalize_metrics`
  - ✅ Fixed variable naming (e → error)

- ✅ **`src/metrics/models.rs`**:
  - ✅ Split `add_language_metrics` into multiple focused functions:
    - ✅ `update_overall_metrics`
    - ✅ `update_language_specific_metrics`
    - ✅ `is_test_file`
    - ✅ `update_test_metrics`
    - ✅ `update_test_language_metrics`
    - ✅ `update_production_metrics`
    - ✅ `update_production_language_metrics`
  - ✅ Parameter passing improved (using references where appropriate)

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