# Claude's Memory File

## Project Overview
CodeAnalyzer is a high-performance, Rust-powered CLI tool designed to analyze codebases. It extracts metrics, detects coding styles, generates AI-ready summaries, maps architecture and dependencies, and more. Built with Rust for memory safety, performance, concurrency, and reliability, it targets developers, AI engineers, DevOps teams, and software architects.

## Codebase Structure
- **src/main.rs**: Entry point with CLI interface using clap for command parsing
- **src/lib.rs**: Re-exports public modules for testing and external use
- **src/analyzer/**: Core analysis functionality
  - **mod.rs**: Main Analyzer struct that orchestrates code metrics and dependency analysis
  - **file_analyzer.rs**: Handles individual file analysis
- **src/metrics/**: Code metrics functionality
  - **collector.rs**: Collects metrics from codebase
  - **models.rs**: Data structures for metrics (CodeMetrics, LanguageMetrics)
  - **reporter.rs**: Formats and displays metrics data
  - **language.rs**: Language detection and line counting
- **src/dependency/**: Dependency analysis 
  - **dependency_analyzer.rs**: Analyzes code dependencies
  - **dependency_graph.rs**: Graph structure representing dependencies
  - **dependency_reporter.rs**: Reports and exports dependency data
- **src/output/**: Output formatting utilities
  - **style.rs**: Console styling and formatting functions
- **src/ai/**: AI integration
  - **mod.rs**: Main AI module with provider trait
  - **config.rs**: Configuration and environment setup
  - **anthropic.rs**: Anthropic Claude models implementation
  - **openai.rs**: OpenAI models implementation
  - **mistral.rs**: Mistral models implementation
  - **factory.rs**: Factory for creating AI providers
- **tests/**: Test cases for all functionality

## Features
- Code metrics: LOC, blank lines, comment lines
- Separate metrics for production and test code
- Language-specific metrics 
- Dependency analysis with graph visualization
- Circular dependency detection
- Export dependency graphs in DOT format
- AI-powered code analysis using different providers and models

## AI Integration
- Support for multiple AI providers: Anthropic, OpenAI, Mistral
- Tiered model selection (low, medium, high) for each provider
- Configurable via environment variables for easy switching
- Provider-specific model mapping:
  - **Low tier**: Smaller, faster, cost-effective models for simple tasks
  - **Medium tier**: Balanced models for regular analysis tasks
  - **High tier**: Powerful models for complex code understanding

### AI Environment Variables
- `AI_PROVIDER`: Selected provider (anthropic, openai, mistral)
- `AI_TIER`: Default model tier (low, medium, high)
- `AI_API_KEY`: API key for the selected provider
- `ANTHROPIC_LOW_MODEL`: Model name for Anthropic low tier
- `ANTHROPIC_MEDIUM_MODEL`: Model name for Anthropic medium tier
- `ANTHROPIC_HIGH_MODEL`: Model name for Anthropic high tier
- `OPENAI_LOW_MODEL`: Model name for OpenAI low tier
- `OPENAI_MEDIUM_MODEL`: Model name for OpenAI medium tier
- `OPENAI_HIGH_MODEL`: Model name for OpenAI high tier
- `MISTRAL_LOW_MODEL`: Model name for Mistral low tier
- `MISTRAL_MEDIUM_MODEL`: Model name for Mistral medium tier
- `MISTRAL_HIGH_MODEL`: Model name for Mistral high tier

## Command Line Interface
- `cargo run -- run [path]`: Full analysis (metrics + dependencies)
- `cargo run -- metrics [path]`: Code metrics only
- `cargo run -- dependencies [path] [--output file.dot]`: Dependency analysis with optional DOT export

## Code Quality Standards
- Follow Clean Code principles:
  - Use meaningful and intention-revealing names
  - Functions should do one thing only and do it well
  - Keep functions small (preferably under 20 lines)
  - Arguments should be few (ideally 0-2, maximum 3)
  - Avoid side effects in functions
  - Don't repeat yourself (DRY)
  - Maintain clear separation of concerns
- Follow existing Rust formatting conventions
- Use idiomatic Rust patterns
- Always add or update unit tests when adding or modifying functionality
- Aim for high test coverage, especially for complex logic
- Each public function/method should have corresponding test cases

## Comments Policy
- NEVER add comments to implementation files - code should be self-documenting
- ONLY add explanatory comments to test files for describing test setup and assertions
- No TODOs, no explanatory comments, no commented-out code in implementation files
- Use meaningful variable and function names instead of comments

- Prefer composition over inheritance
- Implement Default trait for structs with new() methods
- Follow Rust's ownership model correctly
- Use error handling consistently (Result/Option)

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

## Project Commands
- Build: `cargo build`
- Test: `cargo test`
- Format: `cargo fmt`
- Lint: `cargo clippy`
- Run: `cargo run -- [path]`

## CRITICAL: NO AI ATTRIBUTION
- ⚠️ NEVER include "Generated with Claude Code", "Co-Authored-By: Claude", or ANY similar attribution in:
  - Git commits
  - Pull requests
  - Code or comments
  - Commit messages
  - PR descriptions
  - Documentation
  - Any other files or outputs
- ⚠️ NEVER mention Claude, AI, or any form of AI assistance anywhere in the codebase or commit history
- ⚠️ When making git commits, NEVER add any co-author or attribution lines

## Important Notes
- Maintain the modular architecture to allow for future extension
- All public APIs should have clear interfaces and documentation
- Handle errors gracefully with informative messages
