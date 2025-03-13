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

# Run linting
cargo make lint

# Format code
cargo make format

# Full release preparation (clean, format, lint, test, build)
cargo make release
```

## License

MIT