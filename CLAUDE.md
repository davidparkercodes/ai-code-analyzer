# Claude's Memory File

## Project Overview
CodeAnalyzer is a high-performance, Rust-powered CLI tool designed to analyze codebases. It extracts metrics, detects coding styles, generates AI-ready summaries, maps architecture and dependencies, and more. Built with Rust for memory safety, performance, concurrency, and reliability, it targets developers, AI engineers, DevOps teams, and software architects.

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

## Important Notes
- NEVER include "Generated with Claude Code" or similar attribution in git commits, PRs, or any code
- NEVER mention Claude or AI assistance in code, comments, commit messages, or PR descriptions
- Maintain the modular architecture to allow for future extension
- All public APIs should have clear interfaces and documentation
- Handle errors gracefully with informative messages
