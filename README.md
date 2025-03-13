# CodeAnalyzer

A high-performance CLI tool built in Rust that analyzes and extracts insights from software codebases.

## Features

- Recursive codebase scanning
- Code metrics extraction with separate production and test code reports
- Language detection with detailed breakdown
- Dependency analysis and visualization
- Circular dependency detection
- Exclusion of test files from dependency analysis
- Export to DOT format for visualizing dependencies with tools like Graphviz
- AI-powered code analysis with multi-provider support

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/davidparkercodes/code-analyzer.git
cd code-analyzer

# Build and install
cargo install --path .
```

## Usage

```bash
# Run full analysis (metrics + dependencies)
codeanalyzer run /path/to/code

# Get code metrics only
codeanalyzer metrics /path/to/code

# Analyze dependencies only
codeanalyzer dependencies /path/to/code

# Export dependency graph to DOT format for visualization
codeanalyzer dependencies /path/to/code --output deps.dot

# Visualize with Graphviz (if installed)
codeanalyzer dependencies /path/to/code --output deps.dot && dot -Tpng deps.dot -o deps.png
```

## Metrics

The metrics command provides detailed analysis of your codebase:

- **Overall Metrics**: Total files, lines of code, blank lines, and comments
- **Production Code Metrics**: Metrics for non-test files only
- **Test Code Metrics**: Metrics for test files only
- **Language Breakdown**: Statistics for each programming language detected

Test files are identified by common patterns such as:
- Files in test/ or tests/ directories
- Files with _test, test_, or *Test.* in their names
- Files matching common test naming patterns like spec.js

## Dependencies

The dependencies command analyzes import statements and module references:

- **Dependency Graph**: Finds all file dependencies across your codebase
- **Circular Dependencies**: Identifies circular dependencies that might cause issues
- **Top Dependencies**: Shows files with the most connections to other files

Test files are excluded from dependency analysis to give a clearer picture of your production code architecture.

## Cross-Platform Building

This project uses [cargo-make](https://github.com/sagiegurari/cargo-make) to simplify cross-platform builds.

### Prerequisites

1. Install cargo-make:
```bash
cargo install cargo-make
```

2. For cross-compilation, you'll need the appropriate toolchains:
   - For Windows builds on non-Windows: Install the MSVC toolchain
   - For Linux builds on non-Linux: Install the GCC toolchain for Linux

### Build Commands

```bash
# Build for all platforms
cargo make build

# Build for specific platforms
cargo make build-release-macos
cargo make build-release-linux
cargo make build-release-windows

# Run tests
cargo make test

# Run unit tests only
cargo make test-unit

# Run linting
cargo make lint

# Format code
cargo make format

# Install pre-commit hooks (recommended for development)
./scripts/install-hooks.sh

# Full release preparation (clean, format, lint, test, build)
cargo make release
```

## Testing

This project uses Rust's built-in testing framework. Tests are organized into:

- **Unit Tests**: Testing individual components in isolation
- **Integration Tests**: Testing the CLI as a whole (coming soon)

### Running Tests

```bash
# Run all tests
cargo test

# Run all tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Pre-commit Hooks

The project uses pre-commit hooks to ensure code quality before commits. These hooks run:

1. Tests
2. Code formatting check
3. Linting with clippy

To install the pre-commit hooks:

1. Install pre-commit (requires Python):
```bash
pip install pre-commit
```

2. Install the hooks:
```bash
pre-commit install
```

Or simply run the provided script:
```bash
./scripts/install-hooks.sh
```

## AI Integration

CodeAnalyzer supports multiple AI providers to enhance code analysis capabilities:

### Supported Providers
- **Anthropic** (Claude models)
- **OpenAI** (GPT models)
- **Mistral** (Mistral models)

### Model Tiers
Each provider supports three tiers of models:
- **Low**: Smaller, faster models for simple tasks and high throughput
- **Medium**: Balanced models for regular analysis tasks
- **High**: Powerful models for complex code understanding and advanced analysis

### Configuration
AI integration is configured through environment variables:

```bash
# Main configuration
AI_PROVIDER=anthropic     # Choose between: anthropic, openai, mistral
AI_TIER=medium           # Choose between: low, medium, high
AI_API_KEY=your_api_key  # API key for the selected provider

# Provider-specific model selection (optional)
# Anthropic models
ANTHROPIC_LOW_MODEL=claude-3-haiku-20240307
ANTHROPIC_MEDIUM_MODEL=claude-3-sonnet-20240229
ANTHROPIC_HIGH_MODEL=claude-3-opus-20240229

# OpenAI models
OPENAI_LOW_MODEL=gpt-3.5-turbo
OPENAI_MEDIUM_MODEL=gpt-4
OPENAI_HIGH_MODEL=gpt-4-turbo

# Mistral models
MISTRAL_LOW_MODEL=mistral-tiny
MISTRAL_MEDIUM_MODEL=mistral-small
MISTRAL_HIGH_MODEL=mistral-large
```

You can set these in a `.env` file in your project root, or in your system environment.

## License

MIT