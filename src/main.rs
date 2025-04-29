use anyhow::Result;
use clap::Parser;

mod cli;
mod utils;

use cli::{Cli, Commands};
use utils::{generate_command, list_command, tokens_command};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate {
            path,
            output,
            dry_run,
        } => generate_command(path, output, *dry_run),
        Commands::Tokens { path, extension } => tokens_command(path, extension.as_deref()),
        Commands::List { path, detailed } => list_command(path, *detailed),
    }
}
