# Technical Context

## Core Libraries

### codebank
- Used for generating summary documentation from code
- Key functionality: `CodeBank::generate` with `BankStrategy::Summary`
- Example usage:
```rust
use codebank::{Bank, BankStrategy, CodeBank, Result};
use std::path::Path;

let code_bank = CodeBank::try_new()?;
let content = code_bank.generate(
    Path::new("src"),
    BankStrategy::Summary
)?;
```

### tokenizers
- Used for calculating token counts in generated files
- Requires the "http" feature for loading pretrained models
- Example usage:
```rust
use tokenizers::tokenizer::{Result, Tokenizer};

let tokenizer = Tokenizer::from_pretrained("bert-base-cased", None)?;
let encoding = tokenizer.encode("Hey there!", false)?;
```

### toml
- Used for parsing Cargo.toml and Cargo.lock files
- Version 0.8 required

### dirs
- Used for finding home directory and cargo registry locations
- Version 6.0.0 required

### clap
- Used for CLI interface and argument parsing
- Version 4.5.37 with "derive" feature

## Environment Details
- Target platform: macOS (with compatibility for Linux/Windows)
- Cargo registry path pattern: `~/.cargo/registry/src/{registry-id}`
- Dependency path pattern: `{registry-path}/{dependency-name}-{version}`
- Output location: `./.codebank/{dep_name}.md`
