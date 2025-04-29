# DepBank: User Guide

DepBank is a tool that generates code banks and calculates tokens for Rust dependencies. This guide provides detailed information on how to use DepBank effectively.

## Table of Contents

- [Installation](#installation)
- [Commands](#commands)
  - [Generate Command](#generate-command)
  - [Tokens Command](#tokens-command)
  - [List Command](#list-command)
- [Working with Different Project Types](#working-with-different-project-types)
- [Understanding Token Calculations](#understanding-token-calculations)
- [Tips and Best Practices](#tips-and-best-practices)
- [Troubleshooting](#troubleshooting)

## Installation

Install DepBank using one of the following methods:

### From crates.io

```bash
cargo install depbank
```

### From source

```bash
git clone https://github.com/your-username/depbank.git
cd depbank
cargo install --path .
```

## Commands

DepBank provides three main commands: `generate`, `tokens`, and `list`.

### Generate Command

The `generate` command creates code banks for dependencies in a Rust project.

#### Syntax

```bash
depbank generate [OPTIONS]
```

#### Options

- `-p, --path <PATH>`: Path to the project root directory (default: current directory)
- `-o, --output <OUTPUT>`: Output directory for generated code banks (default: .codebank)
- `-d, --dry-run`: Only calculate tokens without generating code banks

#### Examples

Basic usage (current directory, output to .codebank):

```bash
depbank generate
```

Specify project directory:

```bash
depbank generate --path /path/to/my/project
```

Custom output directory:

```bash
depbank generate --output ./docs/dependencies
```

Dry run (only calculate tokens, don't generate files):

```bash
depbank generate --dry-run
```

#### Output

The command will:

1. Find all Cargo.toml files in the project
2. Extract dependency information
3. Resolve exact versions from Cargo.lock
4. Find the local Cargo registry
5. Generate code banks for available dependencies
6. Create a README.md with dependency summaries and token information
7. Calculate and display token counts

#### Generated README.md

The `generate` command creates a README.md file in the output directory with:

- A summary table listing all dependencies with their versions and token counts
- Total token statistics for all code banks
- Information about what code banks are and how to use them

This README.md serves as an index and reference for the generated code banks, making it easier to navigate and understand the generated content.

### Tokens Command

The `tokens` command calculates tokens for files or directories.

#### Syntax

```bash
depbank tokens <PATH> [OPTIONS]
```

#### Arguments

- `<PATH>`: Path to file or directory to analyze (required)

#### Options

- `-e, --extension <EXTENSION>`: Filter by file extension (e.g., "md")

#### Examples

Calculate tokens for a single file:

```bash
depbank tokens README.md
```

Calculate tokens for all files in a directory:

```bash
depbank tokens ./docs
```

Calculate tokens only for Markdown files in a directory:

```bash
depbank tokens ./docs --extension md
```

#### Output

For a single file:
```
README.md: 325 tokens, 2048 bytes
```

For a directory:
```
Token counts for files in ./docs:
readme.md: 325 tokens, 2048 bytes
guide.md: 1205 tokens, 7890 bytes
reference.md: 524 tokens, 3456 bytes

Total: 2054 tokens, 13394 bytes across 3 files
```

### List Command

The `list` command shows dependencies in a Rust project.

#### Syntax

```bash
depbank list [OPTIONS]
```

#### Options

- `-p, --path <PATH>`: Path to the project root directory (default: current directory)
- `-d, --detailed`: Show detailed information including versions

#### Examples

List all dependencies in the current project:

```bash
depbank list
```

List dependencies with detailed information:

```bash
depbank list --detailed
```

List dependencies for a specific project:

```bash
depbank list --path /path/to/my/project
```

#### Output

Simple listing:
```
Found 1 Cargo.toml files
Found 5 unique dependencies:
- anyhow
- clap
- serde
- tokio
- toml
```

Detailed listing:
```
Found 1 Cargo.toml files

Dependency specifications from ./Cargo.toml:
anyhow: 1
clap: { version = "4.5", features = ["derive"] }
serde: { version = "1", features = ["derive"] }
tokio: { version = "1", features = ["full"] }
toml: 0.8

Resolved dependency versions from Cargo.lock:
anyhow: 1.0.75
clap: 4.5.1
serde: 1.0.188
tokio: 1.32.0
toml: 0.8.2
```

## Working with Different Project Types

### Standard Rust Projects

For a standard Rust project with a single Cargo.toml file, use DepBank as follows:

```bash
cd my-project
depbank generate
```

### Workspace Projects

For workspace projects with multiple packages, DepBank will find all Cargo.toml files in the workspace and process dependencies from all of them:

```bash
cd my-workspace
depbank list
# Shows dependencies from all packages in the workspace

depbank generate
# Generates code banks for all dependencies from all packages
```

### Projects with Many Dependencies

For projects with many dependencies, you might want to:

1. Use the `--dry-run` option first to see which dependencies will be processed:
   ```bash
   depbank generate --dry-run
   ```

2. Use a custom output directory to keep things organized:
   ```bash
   depbank generate --output ./docs/dependencies
   ```

## Understanding Token Calculations

DepBank uses a pretrained BERT tokenizer to calculate token counts, similar to how GPT models tokenize text. This helps you understand the token usage when using the generated code banks with AI assistants.

Token counts include:
- All characters, including whitespace
- Special tokens for formatting
- Code syntax highlighting tokens

The token counts are particularly useful for:
- Estimating costs for API usage with AI models
- Understanding the size and complexity of your dependencies
- Optimizing documentation for token efficiency

## Tips and Best Practices

### Optimizing Performance

- When dealing with large projects, target specific subdirectories instead of the entire project:
  ```
