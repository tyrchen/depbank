use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "depbank",
    about = "Generate code banks and calculate tokens for Rust dependencies",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate code banks for dependencies in a Rust project
    Generate {
        /// Path to the project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Output directory for generated code banks
        #[arg(short, long, default_value = ".codebank")]
        output: PathBuf,

        /// Only calculate tokens without generating code banks
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Calculate tokens for files or directories
    Tokens {
        /// Path to file or directory to analyze
        #[arg(required = true)]
        path: PathBuf,

        /// Filter by file extension (e.g., "md")
        #[arg(short, long)]
        extension: Option<String>,
    },

    /// List dependencies in a Rust project
    List {
        /// Path to the project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Show detailed information including versions
        #[arg(short, long)]
        detailed: bool,
    },
}
