use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
enum AppError {
    #[error("Failed to process data: {0}")]
    ProcessError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    debug: bool,
    log_level: String,
    features: Vec<String>,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process data
    Process {
        /// Input file
        #[arg(short, long)]
        input: String,

        /// Output file
        #[arg(short, long)]
        output: String,
    },

    /// Show configuration
    Config,
}

fn main() -> Result<()> {
    #[cfg(feature = "logging")]
    log::info!("Application started");

    let cli = Cli::parse();

    match cli.command {
        Commands::Process { input, output } => {
            println!("Processing data from {} to {}", input, output);
        }
        Commands::Config => {
            let config = Config {
                debug: true,
                log_level: "info".to_string(),
                features: vec!["logging".to_string()],
            };

            println!("Configuration: {:?}", config);
        }
    }

    Ok(())
}
