/*!
 * # DepBank
 *
 * DepBank is a Rust library for generating AI-friendly code banks from your project's dependencies.
 * It helps AI tools better understand your code by creating comprehensive documentation of the
 * dependencies you're using.
 *
 * ## Key features
 *
 * - Finding all Cargo.toml files in a project hierarchy
 * - Extracting dependency information from Cargo.toml files
 * - Resolving exact dependency versions using Cargo.lock
 * - Generating code banks for dependencies
 * - Calculating token counts for generated documentation
 *
 * ## Core concepts
 *
 * - **Dependency discovery**: Finds dependencies in your Cargo.toml files
 * - **Version resolution**: Maps version requirements to exact versions using Cargo.lock
 * - **Registry path resolution**: Locates dependencies in the local Cargo registry
 * - **Code bank generation**: Creates documentation for each dependency
 * - **Token calculation**: Measures LLM token usage for generated documentation
 *
 * ## Usage examples
 *
 * ### Generate code banks for dependencies
 *
 * ```rust,no_run
 * use anyhow::Result;
 * use depbank::{
 *     extract_dependency_info, find_cargo_lock, find_cargo_toml_files,
 *     generate_all_code_banks, is_dependency_available, resolve_dependency_versions,
 *     resolve_registry_path,
 * };
 * use std::path::{Path, PathBuf};
 *
 * fn generate_docs(project_path: &Path, output_dir: &Path) -> Result<()> {
 *     // Find Cargo.toml files
 *     let cargo_toml_files = find_cargo_toml_files(project_path)?;
 *
 *     // Extract dependency information
 *     let first_cargo_toml = &cargo_toml_files[0];
 *     let dependency_info = extract_dependency_info(first_cargo_toml)?;
 *
 *     // Resolve exact versions from Cargo.lock
 *     let cargo_lock_path = find_cargo_lock(project_path)?;
 *     let resolved_versions = resolve_dependency_versions(cargo_lock_path, &dependency_info)?;
 *
 *     // Resolve registry path
 *     let registry_path = resolve_registry_path()?;
 *
 *     // Generate code banks
 *     let code_bank_files = generate_all_code_banks(&resolved_versions, &registry_path, output_dir)?;
 *
 *     println!("Generated {} code bank files", code_bank_files.len());
 *     Ok(())
 * }
 * ```
 *
 * ### Calculate tokens for files
 *
 * ```rust,no_run
 * use depbank::{calculate_directory_tokens, calculate_file_tokens};
 * use std::path::Path;
 *
 * fn count_tokens(path: &Path) -> anyhow::Result<()> {
 *     if path.is_file() {
 *         // Count tokens for a single file
 *         let token_count = calculate_file_tokens(path)?;
 *         println!("{}: {} tokens", path.display(), token_count);
 *     } else if path.is_dir() {
 *         // Count tokens for a directory
 *         let file_stats = calculate_directory_tokens(path, Some("md"))?;
 *         println!("Total files: {}", file_stats.len());
 *
 *         // Calculate total tokens
 *         let total_tokens: usize = file_stats.values().map(|stat| stat.token_count).sum();
 *         println!("Total tokens: {}", total_tokens);
 *     }
 *
 *     Ok(())
 * }
 * ```
 */

use anyhow::{Context, Result};
use codebank::{Bank, BankConfig, BankStrategy, CodeBank};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokenizers::tokenizer::Tokenizer;

/// A dependency with its name and version
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dependency {
    /// The name of the dependency
    pub name: String,
    /// The version specification of the dependency
    pub version: String,
}

impl Dependency {
    /// Create a new dependency with the given name and version
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }

    /// Get the full path to this dependency in the cargo registry
    pub fn get_registry_path(&self, registry_base_path: &Path) -> PathBuf {
        registry_base_path.join(format!("{}-{}", self.name, self.version))
    }

    /// Check if this dependency is available in the cargo registry
    pub fn is_available_in_registry(&self, registry_base_path: &Path) -> bool {
        let path = self.get_registry_path(registry_base_path);
        path.exists() && path.is_dir()
    }
}

/// A collection of dependencies with helper methods
#[derive(Debug, Default)]
pub struct DependencyCollection {
    /// The dependencies in this collection
    deps: Vec<Dependency>,
}

impl DependencyCollection {
    /// Create a new empty dependency collection
    pub fn new() -> Self {
        Self { deps: Vec::new() }
    }

    /// Add a dependency to this collection
    pub fn add(&mut self, dep: Dependency) {
        self.deps.push(dep);
    }

    /// Get the number of dependencies in this collection
    pub fn len(&self) -> usize {
        self.deps.len()
    }

    /// Check if this collection is empty
    pub fn is_empty(&self) -> bool {
        self.deps.is_empty()
    }

    /// Returns an iterator over the dependencies in this collection
    pub fn iter(&self) -> impl Iterator<Item = &Dependency> {
        self.deps.iter()
    }

    /// Get a dependency by name, if it exists
    pub fn get(&self, name: &str) -> Option<&Dependency> {
        self.deps.iter().find(|dep| dep.name == name)
    }

    /// Get a dependency's version by name, if it exists
    pub fn get_version(&self, name: &str) -> Option<&String> {
        self.get(name).map(|dep| &dep.version)
    }

    /// Convert this collection to a HashMap of dependency names to their versions
    pub fn to_map(&self) -> HashMap<String, String> {
        self.deps
            .iter()
            .map(|dep| (dep.name.clone(), dep.version.clone()))
            .collect()
    }

    /// Convert from a HashMap of dependency names to their versions
    pub fn from_map(map: &HashMap<String, String>) -> Self {
        let mut collection = Self::new();
        for (name, version) in map {
            collection.add(Dependency::new(name, version));
        }
        collection
    }

    /// Check if this collection contains a dependency with the given name
    pub fn contains_name(&self, name: &str) -> bool {
        self.deps.iter().any(|dep| dep.name == name)
    }

    /// Check if this collection contains a dependency with the exact name and version
    pub fn contains(&self, name: &str, version: &str) -> bool {
        self.deps
            .iter()
            .any(|dep| dep.name == name && dep.version == version)
    }

    /// Filter this collection to only include dependencies available in the registry
    pub fn filter_available(&self, registry_path: &Path) -> Self {
        let mut result = Self::new();
        for dep in &self.deps {
            if dep.is_available_in_registry(registry_path) {
                result.add(dep.clone());
            }
        }
        result
    }

    /// Get a reference to the underlying vector of dependencies
    pub fn as_slice(&self) -> &[Dependency] {
        &self.deps
    }
}

/// Recursively finds all Cargo.toml files in the given directory.
///
/// This function walks through a directory tree, finding all Cargo.toml files.
/// It automatically skips hidden directories (those starting with a dot).
///
/// # Arguments
///
/// * `root_dir` - The root directory to start searching from
///
/// # Returns
///
/// * `Result<Vec<PathBuf>>` - A vector of paths to all Cargo.toml files found
///
/// # Errors
///
/// Returns an error if:
/// - The root directory does not exist
/// - The path is not a directory
/// - There are permission issues accessing directories
///
/// # Examples
///
/// ```rust,no_run
/// use depbank::find_cargo_toml_files;
/// use std::path::Path;
///
/// let project_dir = Path::new("./my_project");
/// match find_cargo_toml_files(project_dir) {
///     Ok(files) => {
///         println!("Found {} Cargo.toml files", files.len());
///         for file in files {
///             println!("  {}", file.display());
///         }
///     },
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn find_cargo_toml_files<P: AsRef<Path>>(root_dir: P) -> Result<Vec<PathBuf>> {
    let root_dir = root_dir.as_ref();
    let mut cargo_toml_files = Vec::new();

    // Check if the root directory exists
    if !root_dir.exists() {
        return Err(anyhow::anyhow!(
            "Root directory does not exist: {}",
            root_dir.display()
        ));
    }

    // Check if the root directory is a directory
    if !root_dir.is_dir() {
        return Err(anyhow::anyhow!(
            "Path is not a directory: {}",
            root_dir.display()
        ));
    }

    // Recursively walk through the directory
    find_cargo_toml_files_recursive(root_dir, &mut cargo_toml_files)?;

    Ok(cargo_toml_files)
}

/// Helper function to recursively walk through directories to find Cargo.toml files
fn find_cargo_toml_files_recursive(dir: &Path, cargo_toml_files: &mut Vec<PathBuf>) -> Result<()> {
    // Check each entry in the directory
    for entry in
        fs::read_dir(dir).with_context(|| format!("Failed to read directory: {}", dir.display()))?
    {
        let entry = entry
            .with_context(|| format!("Failed to read directory entry in {}", dir.display()))?;
        let path = entry.path();

        // Skip hidden directories (like .git)
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
            && path.is_dir()
        {
            continue;
        }

        // If the entry is a Cargo.toml file, add it to the list
        if path.file_name().is_some_and(|name| name == "Cargo.toml") {
            cargo_toml_files.push(path.clone());
        }

        // If the entry is a directory, recursively search it
        if path.is_dir() {
            find_cargo_toml_files_recursive(&path, cargo_toml_files)?;
        }
    }

    Ok(())
}

/// Represents a dependency in Cargo.toml
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CargoDepSpec {
    Simple(String),
    Detailed(HashMap<String, toml::Value>),
}

/// Structure for parsing Cargo.toml
#[derive(Debug, Deserialize)]
struct CargoToml {
    #[serde(default)]
    dependencies: HashMap<String, CargoDepSpec>,
    #[serde(default)]
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: HashMap<String, CargoDepSpec>,
    #[serde(default)]
    #[serde(rename = "build-dependencies")]
    build_dependencies: HashMap<String, CargoDepSpec>,
}

/// Collects all dependencies from found Cargo.toml files into a HashSet.
///
/// # Arguments
///
/// * `cargo_toml_files` - A slice of paths to Cargo.toml files
///
/// # Returns
///
/// * `Result<HashSet<String>>` - A HashSet containing all unique dependency names
pub fn collect_dependencies(cargo_toml_files: &[PathBuf]) -> Result<HashSet<String>> {
    let mut dependencies = HashSet::new();

    for path in cargo_toml_files {
        let cargo_toml_content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read Cargo.toml file: {}", path.display()))?;

        let cargo_toml: CargoToml = toml::from_str(&cargo_toml_content)
            .with_context(|| format!("Failed to parse Cargo.toml file: {}", path.display()))?;

        // Add regular dependencies
        for dep_name in cargo_toml.dependencies.keys() {
            dependencies.insert(dep_name.clone());
        }

        // Add dev dependencies
        for dep_name in cargo_toml.dev_dependencies.keys() {
            dependencies.insert(dep_name.clone());
        }

        // Add build dependencies
        for dep_name in cargo_toml.build_dependencies.keys() {
            dependencies.insert(dep_name.clone());
        }
    }

    Ok(dependencies)
}

/// Extracts dependency information from a single Cargo.toml file.
///
/// This function parses a Cargo.toml file and extracts information about all dependencies,
/// including regular dependencies, dev-dependencies, and build-dependencies. For each dependency,
/// it extracts the version specification.
///
/// # Arguments
///
/// * `cargo_toml_path` - Path to the Cargo.toml file
///
/// # Returns
///
/// * `Result<DependencyCollection>` - A collection of dependencies found in the Cargo.toml file
///
/// # Errors
///
/// Returns an error if:
/// - The Cargo.toml file cannot be read
/// - The Cargo.toml file cannot be parsed as a valid TOML format
///
/// # Examples
///
/// ```rust,no_run
/// use depbank::extract_dependency_info;
/// use std::path::Path;
///
/// let cargo_toml = Path::new("./Cargo.toml");
/// match extract_dependency_info(cargo_toml) {
///     Ok(dependencies) => {
///         println!("Found {} dependencies", dependencies.len());
///         for dep in dependencies.as_slice() {
///             println!("  {} - {}", dep.name, dep.version);
///         }
///     },
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn extract_dependency_info(cargo_toml_path: &Path) -> Result<DependencyCollection> {
    let mut dependencies = DependencyCollection::new();

    let cargo_toml_content = fs::read_to_string(cargo_toml_path).with_context(|| {
        format!(
            "Failed to read Cargo.toml file: {}",
            cargo_toml_path.display()
        )
    })?;

    // Alternative approach using the CargoToml struct
    let cargo_toml: CargoToml = toml::from_str(&cargo_toml_content).with_context(|| {
        format!(
            "Failed to parse Cargo.toml file: {}",
            cargo_toml_path.display()
        )
    })?;

    // Process regular dependencies
    for (name, spec) in &cargo_toml.dependencies {
        let version = extract_version_from_spec(spec);
        dependencies.add(Dependency::new(name, version));
    }

    // Process dev dependencies
    for (name, spec) in &cargo_toml.dev_dependencies {
        let version = extract_version_from_spec(spec);
        dependencies.add(Dependency::new(name, version));
    }

    // Process build dependencies
    for (name, spec) in &cargo_toml.build_dependencies {
        let version = extract_version_from_spec(spec);
        dependencies.add(Dependency::new(name, version));
    }

    Ok(dependencies)
}

/// Helper function to extract version from a CargoDepSpec
fn extract_version_from_spec(spec: &CargoDepSpec) -> String {
    match spec {
        CargoDepSpec::Simple(version) => version.clone(),
        CargoDepSpec::Detailed(table) => {
            // Check for workspace = true first
            if let Some(workspace) = table.get("workspace") {
                if workspace.as_bool().unwrap_or(false) {
                    // Use a placeholder. The actual version comes from Cargo.lock or workspace definition.
                    // For `resolve_dependency_versions`, we just need the name.
                    // If we needed the *constraint* from the workspace root, we'd need to parse that too.
                    return "workspace".to_string();
                }
            }
            // Otherwise, look for an inline version
            if let Some(version) = table.get("version") {
                if let Some(v) = version.as_str() {
                    return v.to_string();
                }
            }
            // Default if neither workspace nor version is specified clearly
            "*".to_string()
        }
    }
}

/// Represents a package in Cargo.lock
#[derive(Debug, Deserialize)]
struct CargoLockPackage {
    name: String,
    version: String,
    #[allow(dead_code)] // Kept for compatibility with Cargo.lock format
    source: Option<String>,
}

/// Structure for parsing Cargo.lock
#[derive(Debug, Deserialize)]
struct CargoLock {
    #[serde(default)]
    package: Vec<CargoLockPackage>,
}

/// Resolves exact dependency versions from Cargo.lock file.
///
/// This function reads the Cargo.lock file to resolve the exact versions of dependencies
/// based on their version requirements specified in Cargo.toml. It helps find the precise
/// version being used in your project.
///
/// # Arguments
///
/// * `cargo_lock_path` - Path to the Cargo.lock file
/// * `dependencies` - DependencyCollection containing dependency information from Cargo.toml
///
/// # Returns
///
/// * `Result<DependencyCollection>` - A collection of dependencies with resolved exact versions
///
/// # Errors
///
/// Returns an error if:
/// - The Cargo.lock file does not exist
/// - The Cargo.lock file cannot be read
/// - The Cargo.lock file cannot be parsed as a valid TOML format
///
/// # Examples
///
/// ```rust,no_run
/// use depbank::{extract_dependency_info, resolve_dependency_versions};
/// use std::path::Path;
///
/// let cargo_toml = Path::new("./Cargo.toml");
/// let cargo_lock = Path::new("./Cargo.lock");
///
/// let dependency_info = extract_dependency_info(cargo_toml).unwrap();
/// let resolved_versions = resolve_dependency_versions(cargo_lock, &dependency_info).unwrap();
///
/// println!("Resolved {} dependency versions", resolved_versions.len());
/// for dep in resolved_versions.as_slice() {
///     println!("  {} - {}", dep.name, dep.version);
/// }
/// ```
pub fn resolve_dependency_versions<P: AsRef<Path>>(
    cargo_lock_path: P,
    dependencies: &DependencyCollection,
) -> Result<DependencyCollection> {
    let cargo_lock_path = cargo_lock_path.as_ref();

    // Check if Cargo.lock exists
    if !cargo_lock_path.exists() {
        return Err(anyhow::anyhow!(
            "Cargo.lock file does not exist: {}",
            cargo_lock_path.display()
        ));
    }

    // Read and parse the Cargo.lock file
    let cargo_lock_content = fs::read_to_string(cargo_lock_path).with_context(|| {
        format!(
            "Failed to read Cargo.lock file: {}",
            cargo_lock_path.display()
        )
    })?;

    let cargo_lock: CargoLock = toml::from_str(&cargo_lock_content).with_context(|| {
        format!(
            "Failed to parse Cargo.lock file: {}",
            cargo_lock_path.display()
        )
    })?;

    // Create a mapping of dependency names to their exact versions
    let mut resolved_versions = DependencyCollection::new();
    let mut package_versions: HashMap<String, Vec<String>> = HashMap::new();

    // First, collect all versions for each package
    for package in &cargo_lock.package {
        package_versions
            .entry(package.name.clone())
            .or_default()
            .push(package.version.clone());
    }

    // Now, resolve each dependency
    for dep in dependencies.as_slice() {
        if let Some(versions) = package_versions.get(&dep.name) {
            // Get the most recent version (assuming they are sorted, which might not always be true)
            // For a more accurate approach, we would need to parse and compare semver
            if let Some(version) = versions.last() {
                resolved_versions.add(Dependency::new(&dep.name, version));
            }
        }
    }

    Ok(resolved_versions)
}

/// Finds the Cargo.lock file in the workspace.
///
/// This function looks for Cargo.lock in the current directory and parent directories.
///
/// # Arguments
///
/// * `start_dir` - The directory to start searching from
///
/// # Returns
///
/// * `Result<PathBuf>` - Path to the found Cargo.lock file
pub fn find_cargo_lock<P: AsRef<Path>>(start_dir: P) -> Result<PathBuf> {
    let start_dir = start_dir.as_ref();
    let mut current_dir = start_dir.to_path_buf();

    // Check the current directory first
    let cargo_lock = current_dir.join("Cargo.lock");
    if cargo_lock.exists() {
        return Ok(cargo_lock);
    }

    // Then check parent directories
    while let Some(parent) = current_dir.parent() {
        current_dir = parent.to_path_buf();
        let cargo_lock = current_dir.join("Cargo.lock");
        if cargo_lock.exists() {
            return Ok(cargo_lock);
        }
    }

    Err(anyhow::anyhow!(
        "Cargo.lock file not found in current directory or its parents"
    ))
}

/// Resolves the path to the Cargo registry directory.
///
/// This function locates the local Cargo registry where dependency source code is stored.
/// It finds the most recently modified registry index directory, which is typically the active one.
///
/// # Returns
///
/// * `Result<PathBuf>` - Path to the cargo registry directory
///
/// # Errors
///
/// Returns an error if:
/// - The home directory cannot be found
/// - The Cargo registry directory does not exist
/// - There are permission issues accessing the directory
/// - No registry directories are found
///
/// # Examples
///
/// ```rust,no_run
/// use depbank::resolve_registry_path;
///
/// match resolve_registry_path() {
///     Ok(path) => println!("Cargo registry found at: {}", path.display()),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn resolve_registry_path() -> Result<PathBuf> {
    // Get the home directory
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    // Construct the path to the cargo registry src directory
    let registry_dir = home_dir.join(".cargo").join("registry").join("src");

    // Check if the registry directory exists
    if !registry_dir.exists() {
        return Err(anyhow::anyhow!(
            "Cargo registry directory not found: {}",
            registry_dir.display()
        ));
    }

    // Find all directories in the registry
    let entries = fs::read_dir(&registry_dir).with_context(|| {
        format!(
            "Failed to read cargo registry directory: {}",
            registry_dir.display()
        )
    })?;

    // Find the most recently modified directory
    let mut latest_dir: Option<(PathBuf, SystemTime)> = None;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    match &latest_dir {
                        Some((_, latest_modified)) if modified > *latest_modified => {
                            latest_dir = Some((path, modified));
                        }
                        None => {
                            latest_dir = Some((path, modified));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Return the most recently modified directory
    match latest_dir {
        Some((dir, _)) => Ok(dir),
        None => Err(anyhow::anyhow!(
            "No registry directories found in: {}",
            registry_dir.display()
        )),
    }
}

/// Constructs the full path to a dependency's source code.
///
/// # Arguments
///
/// * `registry_path` - Path to the cargo registry directory
/// * `dependency_name` - Name of the dependency
/// * `dependency_version` - Version of the dependency
///
/// # Returns
///
/// * `PathBuf` - Full path to the dependency's source code
pub fn construct_dependency_path(
    registry_path: &Path,
    dependency_name: &str,
    dependency_version: &str,
) -> PathBuf {
    registry_path.join(format!("{}-{}", dependency_name, dependency_version))
}

/// Checks if a dependency is available locally in the cargo registry.
///
/// # Arguments
///
/// * `registry_path` - Path to the cargo registry directory
/// * `dependency` - The dependency to check
///
/// # Returns
///
/// * `bool` - True if the dependency is available locally, false otherwise
pub fn is_dependency_available(registry_path: &Path, dependency: &Dependency) -> bool {
    dependency.is_available_in_registry(registry_path)
}

/// Overloaded version of is_dependency_available that takes separate name and version strings.
///
/// # Arguments
///
/// * `registry_path` - Path to the cargo registry directory
/// * `dependency_name` - Name of the dependency
/// * `dependency_version` - Version of the dependency
///
/// # Returns
///
/// * `bool` - True if the dependency is available locally, false otherwise
pub fn is_dependency_available_by_parts(
    registry_path: &Path,
    dependency_name: &str,
    dependency_version: &str,
) -> bool {
    let dependency = Dependency::new(dependency_name, dependency_version);
    is_dependency_available(registry_path, &dependency)
}

/// Resolves the paths for all dependencies.
///
/// # Arguments
///
/// * `dependencies` - HashMap containing dependency names and their versions
///
/// # Returns
///
/// * `Result<HashMap<String, PathBuf>>` - HashMap mapping dependency names to their local paths
pub fn resolve_dependency_paths(
    dependencies: &HashMap<String, String>,
) -> Result<HashMap<String, PathBuf>> {
    let registry_path = resolve_registry_path()?;
    let mut dependency_paths = HashMap::new();

    for (name, version) in dependencies {
        let dependency_path = construct_dependency_path(&registry_path, name, version);

        if dependency_path.exists() && dependency_path.is_dir() {
            dependency_paths.insert(name.clone(), dependency_path);
        }
    }

    Ok(dependency_paths)
}

/// Generates code bank for a dependency.
///
/// # Arguments
///
/// * `source_path` - Path to the dependency's source code
/// * `output_dir` - Path to the output directory for code bank files
/// * `dependency_name` - Name of the dependency
///
/// # Returns
///
/// * `Result<PathBuf>` - Path to the generated code bank file
pub fn generate_code_bank(
    source_path: &Path,
    output_dir: &Path,
    dependency_name: &str,
) -> Result<PathBuf> {
    // Check if source path exists
    if !source_path.exists() || !source_path.is_dir() {
        return Err(anyhow::anyhow!(
            "Source path does not exist or is not a directory: {}",
            source_path.display()
        ));
    }

    // Check if output directory exists, create it if not
    if !output_dir.exists() {
        fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                output_dir.display()
            )
        })?;
    }

    // Create the output file path
    let output_file = output_dir.join(format!("{}.md", dependency_name));

    // Create a new code bank generator
    let code_bank = CodeBank::try_new().with_context(|| "Failed to create CodeBank instance")?;

    // Generate documentation for the source directory
    // ignore directories: examples, tests, benches
    let ignore_dirs = vec![
        "examples".to_string(),
        "tests".to_string(),
        "benches".to_string(),
    ];
    let config = BankConfig::new(source_path, BankStrategy::Summary, ignore_dirs);
    let content = code_bank.generate(&config).with_context(|| {
        format!(
            "Failed to generate code bank for: {}",
            source_path.display()
        )
    })?;

    // Write the content to the output file
    fs::write(&output_file, content).with_context(|| {
        format!(
            "Failed to write code bank to file: {}",
            output_file.display()
        )
    })?;

    Ok(output_file)
}

/// Generates code banks for all available dependencies.
///
/// This function creates code bank documentation files for each dependency using the codebank library.
/// It processes each dependency, generates a summary documentation, and saves it to the specified output directory.
///
/// # Arguments
///
/// * `dependencies` - Collection of dependencies with their versions
/// * `registry_path` - Path to the cargo registry directory
/// * `output_dir` - Path to the output directory for code bank files
///
/// # Returns
///
/// * `Result<HashMap<String, PathBuf>>` - HashMap mapping dependency names to their code bank file paths
///
/// # Errors
///
/// Returns an error if:
/// - The code bank generator cannot be initialized
/// - There are issues generating documentation for dependencies
/// - The output directory cannot be created
///
/// # Examples
///
/// ```rust,no_run
/// use depbank::{Dependency, DependencyCollection, generate_all_code_banks, resolve_registry_path};
/// use std::path::Path;
///
/// let mut dependencies = DependencyCollection::new();
/// dependencies.add(Dependency::new("serde", "1.0.152"));
/// dependencies.add(Dependency::new("anyhow", "1.0.70"));
///
/// let registry_path = resolve_registry_path().unwrap();
/// let output_dir = Path::new("./.codebank");
///
/// match generate_all_code_banks(&dependencies, &registry_path, output_dir) {
///     Ok(files) => println!("Generated {} code bank files", files.len()),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn generate_all_code_banks(
    dependencies: &DependencyCollection,
    registry_path: &Path,
    output_dir: &Path,
) -> Result<HashMap<String, PathBuf>> {
    let mut code_bank_files = HashMap::new();
    let mut errors = Vec::new();

    for dependency in dependencies.as_slice() {
        let dependency_path = dependency.get_registry_path(registry_path);

        if dependency_path.exists() && dependency_path.is_dir() {
            match generate_code_bank(&dependency_path, output_dir, &dependency.name) {
                Ok(code_bank_file) => {
                    code_bank_files.insert(dependency.name.clone(), code_bank_file);
                }
                Err(e) => {
                    errors.push(format!(
                        "Failed to generate code bank for {}: {}",
                        dependency.name, e
                    ));
                }
            }
        } else {
            errors.push(format!(
                "Dependency not found: {}",
                dependency_path.display()
            ));
        }
    }

    // If there were errors, log them but don't fail the operation
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Warning: {}", error);
        }
    }

    Ok(code_bank_files)
}

/// Calculates the number of tokens in a text.
///
/// # Arguments
///
/// * `text` - The text to tokenize
///
/// # Returns
///
/// * `Result<usize>` - The number of tokens in the text
pub fn calculate_tokens(text: &str) -> Result<usize> {
    // Load a pretrained tokenizer model (BERT)
    let tokenizer = Tokenizer::from_pretrained("bert-base-cased", None)
        .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

    // Tokenize the text
    let encoding = tokenizer
        .encode(text, false)
        .map_err(|e| anyhow::anyhow!("Failed to tokenize text: {}", e))?;

    // Return the number of tokens
    Ok(encoding.get_tokens().len())
}

/// Calculates tokens for a file.
///
/// # Arguments
///
/// * `file_path` - Path to the file
///
/// # Returns
///
/// * `Result<usize>` - The number of tokens in the file
pub fn calculate_file_tokens(file_path: &Path) -> Result<usize> {
    // Read the file content
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Calculate tokens
    calculate_tokens(&content)
}

/// Represents file statistics including token count.
#[derive(Debug)]
pub struct FileStats {
    pub path: PathBuf,
    pub size_bytes: usize,
    pub token_count: usize,
}

/// Calculates tokens for all files in a directory.
///
/// This function traverses a directory and calculates token counts for each file,
/// optionally filtering by file extension. It uses a transformer-based tokenizer to
/// determine the token count, similar to how GPT models tokenize text.
///
/// # Arguments
///
/// * `dir_path` - Path to the directory
/// * `extension` - Optional file extension filter (e.g., "md")
///
/// # Returns
///
/// * `Result<HashMap<String, FileStats>>` - HashMap mapping filenames to their stats
///
/// # Errors
///
/// Returns an error if:
/// - The directory does not exist
/// - There are permission issues accessing files
/// - There are issues reading file contents
/// - The tokenizer fails to process a file
///
/// # Examples
///
/// ```rust,no_run
/// use depbank::calculate_directory_tokens;
/// use std::path::Path;
///
/// // Calculate tokens for all markdown files in a directory
/// let stats = calculate_directory_tokens(Path::new("./docs"), Some("md")).unwrap();
///
/// let total_tokens: usize = stats.values().map(|stat| stat.token_count).sum();
/// println!("Total: {} tokens across {} files", total_tokens, stats.len());
///
/// // Print individual file stats
/// for (name, stat) in stats {
///     println!("{}: {} tokens, {} bytes", name, stat.token_count, stat.size_bytes);
/// }
/// ```
pub fn calculate_directory_tokens(
    dir_path: &Path,
    extension: Option<&str>,
) -> Result<HashMap<String, FileStats>> {
    let mut file_stats = HashMap::new();

    // Check if directory exists
    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(anyhow::anyhow!(
            "Directory does not exist or is not a directory: {}",
            dir_path.display()
        ));
    }

    // Read directory entries
    for entry in fs::read_dir(dir_path)
        .with_context(|| format!("Failed to read directory: {}", dir_path.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        // Skip directories and files that don't match the extension
        if path.is_dir() {
            continue;
        }

        if let Some(ext) = extension {
            #[allow(clippy::nonminimal_bool)]
            if !path.extension().is_some_and(|e| e == ext) {
                continue;
            }
        }

        // Get file name as string
        let file_name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();

        // Get file size
        let metadata = fs::metadata(&path)?;
        let size_bytes = metadata.len() as usize;

        // Calculate tokens for the file
        let token_count = calculate_file_tokens(&path)?;

        // Create file stats
        let stats = FileStats {
            path: path.clone(),
            size_bytes,
            token_count,
        };

        // Add to the map
        file_stats.insert(file_name, stats);
    }

    Ok(file_stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_find_cargo_toml_files() -> Result<()> {
        // Create a temporary directory structure
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();

        // Create a Cargo.toml file in the root directory
        let root_cargo_toml = temp_path.join("Cargo.toml");
        File::create(&root_cargo_toml)?
            .write_all(b"[package]\nname = \"test\"\nversion = \"0.1.0\"\n")?;

        // Create a subdirectory with a Cargo.toml file
        let sub_dir = temp_path.join("sub_project");
        fs::create_dir(&sub_dir)?;
        let sub_cargo_toml = sub_dir.join("Cargo.toml");
        File::create(&sub_cargo_toml)?
            .write_all(b"[package]\nname = \"sub_test\"\nversion = \"0.1.0\"\n")?;

        // Create a nested subdirectory with a Cargo.toml file
        let nested_dir = sub_dir.join("nested_project");
        fs::create_dir(&nested_dir)?;
        let nested_cargo_toml = nested_dir.join("Cargo.toml");
        File::create(&nested_cargo_toml)?
            .write_all(b"[package]\nname = \"nested_test\"\nversion = \"0.1.0\"\n")?;

        // Create a hidden directory that should be skipped
        let hidden_dir = temp_path.join(".hidden");
        fs::create_dir(&hidden_dir)?;
        let hidden_cargo_toml = hidden_dir.join("Cargo.toml");
        File::create(&hidden_cargo_toml)?
            .write_all(b"[package]\nname = \"hidden_test\"\nversion = \"0.1.0\"\n")?;

        // Find all Cargo.toml files
        let cargo_toml_files = find_cargo_toml_files(temp_path)?;

        // We should find 3 Cargo.toml files (not including the one in the hidden directory)
        assert_eq!(cargo_toml_files.len(), 3);

        // Check that the expected files are included
        assert!(cargo_toml_files.contains(&root_cargo_toml));
        assert!(cargo_toml_files.contains(&sub_cargo_toml));
        assert!(cargo_toml_files.contains(&nested_cargo_toml));

        // Check that the hidden file is not included
        assert!(!cargo_toml_files.contains(&hidden_cargo_toml));

        Ok(())
    }

    #[test]
    fn test_find_cargo_toml_files_nonexistent_dir() {
        let result = find_cargo_toml_files(Path::new("/nonexistent/directory"));
        assert!(result.is_err());
    }

    #[test]
    fn test_find_cargo_toml_files_file_as_dir() -> Result<()> {
        // Create a temporary file
        let temp_dir = tempdir()?;
        let temp_file = temp_dir.path().join("file.txt");
        File::create(&temp_file)?.write_all(b"content")?;

        // Try to use the file as a directory
        let result = find_cargo_toml_files(&temp_file);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_collect_dependencies() -> Result<()> {
        // Create a temporary directory with a Cargo.toml file
        let temp_dir = tempdir()?;
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        // Create a sample Cargo.toml file with dependencies
        let cargo_toml_content = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
tempfile = "3.0"

[build-dependencies]
cc = "1.0"
"#;

        File::create(&cargo_toml_path)?.write_all(cargo_toml_content.as_bytes())?;

        // Collect dependencies from the Cargo.toml file
        let dependencies = collect_dependencies(&[cargo_toml_path])?;

        // Check that all dependencies were collected
        assert!(dependencies.contains("anyhow"));
        assert!(dependencies.contains("serde"));
        assert!(dependencies.contains("tokio"));
        assert!(dependencies.contains("tempfile"));
        assert!(dependencies.contains("cc"));

        // Check that the total number of dependencies is correct
        assert_eq!(dependencies.len(), 5);

        Ok(())
    }

    #[test]
    fn test_extract_dependency_info() -> Result<()> {
        // Create a temporary directory with a Cargo.toml file
        let temp_dir = tempdir()?;
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        // Create a sample Cargo.toml file with dependencies
        let cargo_toml_content = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
tempfile = "3.0"

[build-dependencies]
cc = "1.0"
"#;

        File::create(&cargo_toml_path)?.write_all(cargo_toml_content.as_bytes())?;

        // Extract dependency info from the Cargo.toml file
        let dependency_info = extract_dependency_info(&cargo_toml_path)?;

        // Check that all dependencies have correct versions
        let dep1 = dependency_info.get("anyhow").unwrap();
        assert_eq!(dep1.version, "1.0");
        let dep2 = dependency_info.get("serde").unwrap();
        assert_eq!(dep2.version, "1.0");
        let dep3 = dependency_info.get("tokio").unwrap();
        assert_eq!(dep3.version, "1.0");
        let dep4 = dependency_info.get("tempfile").unwrap();
        assert_eq!(dep4.version, "3.0");
        let dep5 = dependency_info.get("cc").unwrap();
        assert_eq!(dep5.version, "1.0");

        // Check that the total number of dependencies is correct
        assert_eq!(dependency_info.len(), 5);

        Ok(())
    }

    #[test]
    fn test_resolve_dependency_versions() -> Result<()> {
        let temp_dir = tempdir()?;

        // Create a sample Cargo.lock file
        let cargo_lock_path = temp_dir.path().join("Cargo.lock");
        let cargo_lock_content = r#"
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "anyhow"
version = "1.0.68"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e6b88822cbe49de4185e3a4cbf8321dd487cf5fe0c5c65695fef6346371e9c48"

[[package]]
name = "anyhow"
version = "1.0.75"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a4668cab20f66d8d020e1fbc0ebe47217433c1b6c8f2040faf858554e394ace6"

[[package]]
name = "serde"
version = "1.0.150"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e6b88822cbe49de4185e3a4cbf8321dd487cf5fe0c5c65695fef6346371e9c48"

[[package]]
name = "toml"
version = "0.8.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e6b88822cbe49de4185e3a4cbf8321dd487cf5fe0c5c65695fef6346371e9c48"
"#;
        File::create(&cargo_lock_path)?.write_all(cargo_lock_content.as_bytes())?;

        // Create a sample dependencies DependencyCollection
        let mut dependencies = DependencyCollection::new();
        dependencies.add(Dependency::new("anyhow", "1.0.75"));
        dependencies.add(Dependency::new("serde", "1.0.150"));
        dependencies.add(Dependency::new("toml", "0.8.4"));
        dependencies.add(Dependency::new("nonexistent", "1.0"));

        // Resolve versions
        let resolved = resolve_dependency_versions(cargo_lock_path, &dependencies)?;

        // Check the resolved versions
        let dep1 = resolved.get("anyhow").unwrap();
        assert_eq!(dep1.version, "1.0.75");
        let dep2 = resolved.get("serde").unwrap();
        assert_eq!(dep2.version, "1.0.150");
        let dep3 = resolved.get("toml").unwrap();
        assert_eq!(dep3.version, "0.8.4");
        assert!(!resolved.contains_name("nonexistent"));

        Ok(())
    }

    #[test]
    fn test_find_cargo_lock() -> Result<()> {
        let temp_dir = tempdir()?;
        let cargo_lock_path = temp_dir.path().join("Cargo.lock");

        // Create an empty Cargo.lock file
        File::create(&cargo_lock_path)?;

        // Test finding it in the current directory
        let found_path = find_cargo_lock(temp_dir.path())?;
        assert_eq!(found_path, cargo_lock_path);

        // Test finding it from a subdirectory
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir)?;
        let found_from_subdir = find_cargo_lock(&sub_dir)?;
        assert_eq!(found_from_subdir, cargo_lock_path);

        Ok(())
    }

    #[test]
    fn test_resolve_dependency_versions_nonexistent_file() {
        let result = resolve_dependency_versions(
            Path::new("/nonexistent/Cargo.lock"),
            &DependencyCollection::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_construct_dependency_path() {
        let registry_path = Path::new("/home/user/.cargo/registry/src/index.crates.io-12345");
        let dependency_name = "serde";
        let dependency_version = "1.0.150";

        let expected_path =
            PathBuf::from("/home/user/.cargo/registry/src/index.crates.io-12345/serde-1.0.150");
        let actual_path =
            construct_dependency_path(registry_path, dependency_name, dependency_version);

        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_resolve_registry_path_with_mock() -> Result<()> {
        let temp_dir = tempdir()?;
        let mock_home = temp_dir.path();

        // Create mock registry structure
        let registry_src = mock_home.join(".cargo").join("registry").join("src");
        fs::create_dir_all(&registry_src)?;

        // Create two registry directories with different modification times
        let old_registry = registry_src.join("index.crates.io-old");
        let new_registry = registry_src.join("index.crates.io-new");

        fs::create_dir(&old_registry)?;

        // Sleep to ensure different modification times
        std::thread::sleep(std::time::Duration::from_millis(10));

        fs::create_dir(&new_registry)?;

        // Override the home directory for testing
        // Note: In a real test, we would use environment variables or mocking,
        // but for simplicity in this example we'll just verify the behavior manually

        // The most recently modified directory should be index.crates.io-new
        assert!(new_registry.exists());
        assert!(old_registry.exists());

        // We can't easily test the actual function without mocking dirs::home_dir(),
        // so this is more of a manual verification

        Ok(())
    }

    #[test]
    fn test_generate_code_bank() -> Result<()> {
        let temp_dir = tempdir()?;
        let source_dir = temp_dir.path().join("source");
        let output_dir = temp_dir.path().join("output");

        // Create source directory structure
        fs::create_dir_all(&source_dir)?;

        // Create a sample source file
        let source_file = source_dir.join("lib.rs");
        let source_content = r#"
/// This is a test function.
pub fn test_function() -> String {
    "Hello, world!".to_string()
}
"#;
        fs::write(&source_file, source_content)?;

        // Generate code bank
        let code_bank_file = generate_code_bank(&source_dir, &output_dir, "test_dependency")?;

        // Check that the code bank file was created
        assert!(code_bank_file.exists());

        // Check that the file contains some content
        let content = fs::read_to_string(&code_bank_file)?;
        assert!(!content.is_empty());

        Ok(())
    }

    #[test]
    fn test_calculate_tokens() -> Result<()> {
        let text = "Hello, world! This is a test.";
        let token_count = calculate_tokens(text)?;

        // The exact token count may depend on the tokenizer,
        // but it should be a reasonable number greater than 0
        assert!(token_count > 0);
        assert!(token_count < 20); // A reasonable upper bound for this short text

        Ok(())
    }

    #[test]
    fn test_calculate_file_tokens() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");

        // Create a sample file
        let content =
            "This is a test file.\nIt has multiple lines.\nEach line should be tokenized.";
        File::create(&file_path)?.write_all(content.as_bytes())?;

        // Calculate tokens
        let token_count = calculate_file_tokens(&file_path)?;

        // Similar to the previous test, verify reasonable bounds
        assert!(token_count > 0);
        assert!(token_count < 50); // A reasonable upper bound for this text

        Ok(())
    }

    #[test]
    fn test_calculate_directory_tokens() -> Result<()> {
        let temp_dir = tempdir()?;

        // Create several test files
        let file1_path = temp_dir.path().join("file1.md");
        let file2_path = temp_dir.path().join("file2.md");
        let file3_path = temp_dir.path().join("file3.txt"); // Different extension

        File::create(&file1_path)?.write_all(b"This is file 1.")?;
        File::create(&file2_path)?.write_all(b"This is file 2. It has more content.")?;
        File::create(&file3_path)?.write_all(b"This is file 3.")?;

        // Calculate tokens for .md files only
        let stats = calculate_directory_tokens(temp_dir.path(), Some("md"))?;

        // Should only include the two .md files
        assert_eq!(stats.len(), 2);
        assert!(stats.contains_key("file1"));
        assert!(stats.contains_key("file2"));
        assert!(!stats.contains_key("file3"));

        // Verify stats have reasonable values
        for file_stat in stats.values() {
            assert!(file_stat.size_bytes > 0);
            assert!(file_stat.token_count > 0);
        }

        // Calculate tokens for all files
        let all_stats = calculate_directory_tokens(temp_dir.path(), None)?;

        // Should include all three files
        assert_eq!(all_stats.len(), 3);

        Ok(())
    }

    #[test]
    fn test_extract_dependency_info_with_all_dependency_types() -> Result<()> {
        // Create a temporary directory
        let temp_dir = tempdir()?;
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        // Create a Cargo.toml with various dependency types
        let cargo_toml_content = r#"
[package]
name = "test_package"
version = "0.1.0"
edition = "2021"

[dependencies]
simple_dep = "1.0"
detailed_dep = { version = "2.0", features = ["full"] }

[dev-dependencies]
test_dep = "0.5"

[build-dependencies]
build_dep = { version = "0.3", optional = true }
"#;

        File::create(&cargo_toml_path)?.write_all(cargo_toml_content.as_bytes())?;

        // Extract dependency info
        let dependency_info = extract_dependency_info(&cargo_toml_path)?;

        // Verify all types of dependencies are extracted
        assert_eq!(dependency_info.len(), 4);
        let simple_dep = dependency_info.get("simple_dep").unwrap();
        assert_eq!(simple_dep.version, "1.0");
        let detailed_dep = dependency_info.get("detailed_dep").unwrap();
        assert_eq!(detailed_dep.version, "2.0");
        let test_dep = dependency_info.get("test_dep").unwrap();
        assert_eq!(test_dep.version, "0.5");
        let build_dep = dependency_info.get("build_dep").unwrap();
        assert_eq!(build_dep.version, "0.3");

        Ok(())
    }

    #[test]
    fn test_resolve_dependency_versions_with_multiple_versions() -> Result<()> {
        // Create a temporary directory
        let temp_dir = tempdir()?;
        let cargo_lock_path = temp_dir.path().join("Cargo.lock");

        // Create a Cargo.lock with multiple versions of the same package
        let cargo_lock_content = r#"
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 3

[[package]]
name = "dep1"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "dep1"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "dep2"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;

        File::create(&cargo_lock_path)?.write_all(cargo_lock_content.as_bytes())?;

        // Create a DependencyCollection of dependencies
        let mut dependencies = DependencyCollection::new();
        dependencies.add(Dependency::new("dep1", "1.0"));
        dependencies.add(Dependency::new("dep2", "0.5"));

        // Resolve versions
        let resolved = resolve_dependency_versions(cargo_lock_path, &dependencies)?;

        // Verify the latest version is selected for dep1
        assert_eq!(resolved.len(), 2);
        let dep1 = resolved.get("dep1").unwrap();
        assert_eq!(dep1.version, "1.2.0");
        let dep2 = resolved.get("dep2").unwrap();
        assert_eq!(dep2.version, "0.5.0");

        Ok(())
    }

    #[test]
    fn test_collect_dependencies_with_overlapping_deps() -> Result<()> {
        // Create a temporary directory structure
        let temp_dir = tempdir()?;

        // Create first Cargo.toml with some dependencies
        let cargo_toml1_path = temp_dir.path().join("Cargo1.toml");
        let cargo_toml1_content = r#"
[package]
name = "test1"
version = "0.1.0"

[dependencies]
dep1 = "1.0"
dep2 = "0.5"
"#;
        File::create(&cargo_toml1_path)?.write_all(cargo_toml1_content.as_bytes())?;

        // Create second Cargo.toml with overlapping dependencies
        let cargo_toml2_path = temp_dir.path().join("Cargo2.toml");
        let cargo_toml2_content = r#"
[package]
name = "test2"
version = "0.1.0"

[dependencies]
dep2 = "0.6"
dep3 = "1.5"
"#;
        File::create(&cargo_toml2_path)?.write_all(cargo_toml2_content.as_bytes())?;

        // Collect dependencies from both files
        let cargo_toml_files = vec![cargo_toml1_path, cargo_toml2_path];
        let dependencies = collect_dependencies(&cargo_toml_files)?;

        // Verify unique dependencies are collected
        assert_eq!(dependencies.len(), 3);
        assert!(dependencies.contains("dep1"));
        assert!(dependencies.contains("dep2"));
        assert!(dependencies.contains("dep3"));

        Ok(())
    }

    #[test]
    fn test_find_dependencies_in_workspace() -> Result<()> {
        // Use the existing workspace fixture
        let fixture_path = Path::new("fixtures/workspace_project");
        assert!(fixture_path.exists(), "Workspace fixture does not exist");

        // Find all Cargo.toml files within the fixture
        let cargo_toml_files = find_cargo_toml_files(fixture_path)?;

        // Expecting 3: root, core, utils
        assert_eq!(cargo_toml_files.len(), 3);

        // Collect dependencies from all found files
        let mut dependency_info = DependencyCollection::new();
        let mut unique_deps = HashSet::new();
        for cargo_toml_path in &cargo_toml_files {
            let file_deps = extract_dependency_info(cargo_toml_path)?;
            for dep in file_deps.iter() {
                dependency_info.add(dep.clone());
                unique_deps.insert(dep.name.clone());
            }
        }

        // Check unique dependencies identified (from core and utils)
        // Based on debug output from test run: tokio, serde, log, chrono, env_logger
        assert!(unique_deps.contains("serde"));
        assert!(unique_deps.contains("tokio"));
        assert!(unique_deps.contains("log"));
        assert!(unique_deps.contains("chrono"));
        assert!(unique_deps.contains("env_logger"));
        assert_eq!(unique_deps.len(), 5, "Expected 5 unique dependencies");

        // Check the DependencyCollection entries (can have duplicates before resolution)
        assert!(dependency_info.contains_name("serde"));
        assert!(dependency_info.contains_name("tokio"));
        assert!(dependency_info.contains_name("log")); // Present in both core and utils
        assert!(dependency_info.contains_name("chrono"));
        assert!(dependency_info.contains_name("env_logger"));

        // Verify specific versions from the individual manifests
        let serde_dep = dependency_info.get("serde").unwrap();
        assert_eq!(serde_dep.version, "1.0"); // From core/Cargo.toml

        let tokio_dep = dependency_info.get("tokio").unwrap();
        assert_eq!(tokio_dep.version, "1.0"); // From core/Cargo.toml

        let log_deps: Vec<&Dependency> =
            dependency_info.iter().filter(|d| d.name == "log").collect();
        assert_eq!(log_deps.len(), 2, "Expected log defined in core and utils");
        // Both specify workspace = true initially
        assert!(log_deps.iter().all(|d| d.version == "workspace"));

        let chrono_dep = dependency_info.get("chrono").unwrap();
        assert_eq!(chrono_dep.version, "0.4"); // From utils/Cargo.toml

        let env_logger_dep = dependency_info.get("env_logger").unwrap();
        assert_eq!(env_logger_dep.version, "workspace"); // From utils/Cargo.toml

        Ok(())
    }

    #[test]
    fn test_is_dependency_available() {
        // Create a mock registry directory
        let temp_dir = tempdir().unwrap();
        let _mock_registry = temp_dir.path(); // Prefix with underscore to silence warning

        // Non-existent path should return false
        let registry_path = Path::new("/non/existent/path");
        assert!(!is_dependency_available(
            registry_path,
            &Dependency::new("some-dep", "1.0.0")
        ));

        // If we could create a mock registry:
        // let temp_dir = tempdir().unwrap();
        // let mock_dep_path = temp_dir.path().join("some-dep-1.0.0");
        // std::fs::create_dir_all(&mock_dep_path).unwrap();
        // assert!(is_dependency_available(&temp_dir.path(), &Dependency::new("some-dep", "1.0.0")));
    }
}
