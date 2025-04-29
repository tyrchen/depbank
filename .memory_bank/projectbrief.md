# DepBank - Project Brief

## Overview
DepBank is a Rust CLI application that helps generate code bank files for each dependency to assist AI coding.

## Project Objectives
- Parse Cargo.toml files to identify dependencies
- Extract version information from Cargo.lock
- Locate dependency source code in cargo registry
- Generate code bank summaries for each dependency
- Calculate token counts for generated files

## Technical Stack
- **Language**: Rust
- **CLI Framework**: clap
- **Documentation Generator**: codebank
- **Token Counter**: tokenizers

## Key Dependencies
```toml
[dependencies]
anyhow = "1"
clap = { version = "4.5.37", features = ["derive"] }
codebank = "0.4.0"
dirs = "6.0.0"
serde = { version = "1", features = ["derive"] }
tokenizers = { version = "0.21.1", features = ["http"] }
toml = "0.8"
```

## High-Level Process Flow
1. Recursively find all Cargo.toml files in a given directory
2. Parse dependencies into a hashset
3. Parse Cargo.lock to retrieve dependency versions
4. Resolve registry path for each dependency
5. Generate code bank summaries using the codebank library
6. Calculate token counts for each generated file
