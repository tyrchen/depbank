# System Patterns

## Architecture

The DepBank system follows a sequential processing pipeline:

```mermaid
graph TD
    Input["Input: Root Directory"] --> FindToml["Find Cargo.toml Files"]
    FindToml --> ParseDeps["Parse Dependencies"]
    ParseDeps --> FindLock["Find & Parse Cargo.lock"]
    FindLock --> ResolveVersions["Resolve Dependency Versions"]
    ResolveVersions --> FindRegistry["Find Cargo Registry Path"]
    FindRegistry --> ResolvePaths["Resolve Dependency Paths"]
    ResolvePaths --> GenerateBank["Generate Code Bank Files"]
    GenerateBank --> CalcTokens["Calculate Token Counts"]
    CalcTokens --> Output["Output: Generated Files & Stats"]

    style FindToml fill:#d0e0ff,stroke:#0066cc
    style ParseDeps fill:#d0e0ff,stroke:#0066cc
    style FindLock fill:#d0e0ff,stroke:#0066cc
    style ResolveVersions fill:#d0e0ff,stroke:#0066cc
    style FindRegistry fill:#ffe6cc,stroke:#ff9933
    style ResolvePaths fill:#ffe6cc,stroke:#ff9933
    style GenerateBank fill:#d5f5d5,stroke:#00cc66
    style CalcTokens fill:#d5f5d5,stroke:#00cc66
```

## Data Structures

- **Dependencies Hashset**: Stores dependency names from Cargo.toml
- **Version Hashmap**: Maps dependency names to their versions from Cargo.lock
- **Path Hashmap**: Maps dependency names to their full filesystem paths

## Key Algorithms

1. **Recursive File Finding**: To locate all Cargo.toml files
2. **Registry Path Resolution**: Find latest registry by modification date
3. **Dependency Path Construction**: Combine registry path with dependency info
4. **Code Bank Generation**: Use codebank library with summary strategy

## File Organization

- **Input**: Cargo.toml and Cargo.lock
- **External Resources**: Cargo registry (~/.cargo/registry/src)
- **Output**: Generated code bank files (./.codebank/{dep_name}.md)

## Error Handling

Use anyhow for error propagation with descriptive messages:
- Missing Cargo.toml/Cargo.lock files
- Registry not found
- Dependency source code not available
- Code bank generation failures
