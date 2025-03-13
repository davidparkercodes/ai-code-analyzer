# CodeAnalyzer

A high-performance CLI tool built in Rust that analyzes and extracts insights from software codebases.

## Features

- Recursive codebase scanning
- Code metrics extraction
- Language detection
- More features coming soon!

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
# Run basic file analysis
codeanalyzer run /path/to/code

# Get code metrics
codeanalyzer metrics /path/to/code
```

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

## License

MIT