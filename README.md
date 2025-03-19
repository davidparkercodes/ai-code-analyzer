# AI CodeAnalyzer

A high-performance CLI tool built in Rust that analyzes and extracts insights from software codebases using AI.

## Features

- Recursive codebase scanning
- Code metrics extraction with separate production and test code reports
- Language detection with detailed breakdown
- Dependency analysis and visualization
- Circular dependency detection
- Exclusion of test files from dependency analysis
- Comment deletion for Rust, Python, and C#/.NET code files
- Export to DOT format for visualizing dependencies with tools like Graphviz
- Architecture diagram generation in multiple formats (DOT, PlantUML, Mermaid, C4)
- AI-powered code analysis with multi-provider support

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/davidparkercodes/ai-code-analyzer.git
cd ai-code-analyzer

# Build and install
cargo install --path .
```

## Usage

```bash
# Run full analysis (metrics + dependencies)
aicodeanalyzer run /path/to/code

# Get code metrics only
aicodeanalyzer metrics /path/to/code

# Analyze dependencies only
aicodeanalyzer dependencies /path/to/code

# Export dependency graph to DOT format for visualization
aicodeanalyzer dependencies /path/to/code --output deps.dot

# Visualize with Graphviz (if installed)
aicodeanalyzer dependencies /path/to/code --output deps.dot && dot -Tpng deps.dot -o deps.png

# Generate architecture diagrams in different formats
aicodeanalyzer architecture-diagram /path/to/code --format dot
aicodeanalyzer architecture-diagram /path/to/code --format plantuml --group-by-module
aicodeanalyzer architecture-diagram /path/to/code --format mermaid --detail high
aicodeanalyzer architecture-diagram /path/to/code --format c4 --focus src/core

# Delete comments from code files
aicodeanalyzer delete-comments /path/to/code --language rust
aicodeanalyzer delete-comments /path/to/code --language python
aicodeanalyzer delete-comments /path/to/code --language csharp
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

## Architecture Diagrams

The architecture-diagram command generates visual representations of your codebase structure:

```bash
aicodeanalyzer architecture-diagram /path/to/code [options]
```

### Options

- `--format <format>`: Diagram format (dot, plantuml, mermaid, c4, svg) [default: dot]
- `--detail <level>`: Detail level (high, medium, low) [default: medium]
- `--group-by-module`: Group files by their parent module/directory
- `--include-tests`: Include test files in the architecture diagram
- `--focus <path>`: Focus on a specific module or directory
- `--output-path <path>`: Custom output file path
- `--no-output`: Display diagram in console instead of saving to file
- `--no-parallel`: Disable parallel processing

### Supported Formats

- **DOT**: Standard GraphViz format, great for complex visualizations
- **PlantUML**: Text-based UML diagram format with multiple rendering options
- **Mermaid**: JavaScript-based diagramming tool that works in Markdown
- **C4**: Architecture diagram format focusing on containers and components
- **SVG**: Vector graphic format that can be imported into LucidChart and other diagramming tools

### LucidChart Integration

The SVG format provides a direct path to import architecture diagrams into LucidChart:

```bash
# Generate an SVG diagram
aicodeanalyzer architecture-diagram /path/to/code --format svg

# The tool will provide instructions for importing into LucidChart:
# 1. Open LucidChart and create a new diagram
# 2. Click on File → Import → SVG
# 3. Select the generated SVG file
```

Note: While the SVG generation is integrated into the application, you still need to install the Graphviz library on your system:
- For MacOS: `brew install graphviz`
- For Ubuntu/Debian: `sudo apt-get install graphviz`
- For Windows: `winget install graphviz`

After generating a diagram, the tool provides instructions for rendering it using the appropriate tools.

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

AI CodeAnalyzer supports multiple AI providers to enhance code analysis capabilities:

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

# Provider API keys - at minimum, set the key for your preferred provider
ANTHROPIC_API_KEY=your_anthropic_api_key
OPENAI_API_KEY=your_openai_api_key
MISTRAL_API_KEY=your_mistral_api_key

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