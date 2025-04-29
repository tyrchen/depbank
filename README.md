![](https://github.com/tyrchen/rust-lib-template/workflows/build/badge.svg)

# DepBank

DepBank is a Rust CLI tool that helps you generate AI-friendly code banks for your project's dependencies. It automatically parses your Cargo.toml files, resolves the exact versions from Cargo.lock, and generates Markdown documentation for each dependency using the codebank library. Additionally, it calculates token counts to help you understand the size and complexity of your dependency documentation.

## Features

- Recursively finds all Cargo.toml files in your project
- Parses dependencies from Cargo.toml files (regular, dev, and build dependencies)
- Resolves exact dependency versions from Cargo.lock
- Generates summary documentation for each dependency as Markdown files
- Creates a README.md in the output directory with dependency summaries and token information
- Calculates token counts for the generated documentation
- Lists all dependencies with optional detailed information
- Works with workspaces and complex project structures

## Installation

```bash
# From crates.io
cargo install depbank

# Or from source
git clone https://github.com/your-username/depbank.git
cd depbank
cargo install --path .
```

## Usage

DepBank provides several subcommands:

```bash
# Display help
depbank --help

# Generate code banks for dependencies in a project
depbank generate [OPTIONS]

# Calculate tokens for files or directories
depbank tokens <PATH> [OPTIONS]

# List dependencies in a project
depbank list [OPTIONS]
```

### Generate Command

The `generate` command creates code banks for all dependencies in a Rust project:

```bash
# Generate code banks using default options (current directory, output to .codebank)
depbank generate

# Generate for a specific project directory
depbank generate --path /path/to/project

# Use a custom output directory
depbank generate --output ./docs/dependencies

# Dry run (calculate tokens without generating files)
depbank generate --dry-run
```

The command creates a README.md file in the output directory (.codebank by default) containing:
- A summary table of all dependencies with their versions and token counts
- Total token statistics for all code banks
- Information about what code banks are and how to use them

### Tokens Command

The `tokens` command calculates token counts for files or directories:

```bash
# Calculate tokens for a file
depbank tokens path/to/file.md

# Calculate tokens for all files in a directory
depbank tokens ./docs

# Filter by file extension
depbank tokens ./docs --extension md
```

### List Command

The `list` command shows dependencies in a Rust project:

```bash
# List all dependencies
depbank list

# List dependencies for a specific project
depbank list --path /path/to/project

# Show detailed dependency information including versions
depbank list --detailed
```

## Examples

### Generating Code Banks for a Project

```bash
$ cd my-rust-project
$ depbank generate
Analyzing project...
Found 1 Cargo.toml files
Found 5 dependencies
Found Cargo.lock
Resolved 5 versions
5/5 dependencies available locally
Generating code banks...
Generated 5 code bank files

Summary:
- Generated 5 code bank files
- Total tokens: 88200
- Added README.md with summary and token information
- Output directory: .codebank
```

### Listing Dependencies with Details

```bash
$ depbank list --detailed
Found 1 Cargo.toml files

Dependency specifications from ./Cargo.toml:
anyhow: 1
clap: { version = "4.5", features = ["derive"] }
serde: { version = "1", features = ["derive"] }
tokio: { version = "1", features = ["rt", "macros"] }
toml: 0.8

Resolved dependency versions from Cargo.lock:
anyhow: 1.0.75
clap: 4.5.1
serde: 1.0.188
tokio: 1.32.0
toml: 0.8.2
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is distributed under the terms of the MIT license.

See [LICENSE.md](LICENSE.md) for details.

Copyright 2025 Tyr Chen
