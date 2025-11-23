use anyhow::Result;
use clap::Parser;

// Declare our modules
mod cli;
mod cgroups;
mod container;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Run { cmd, args } => container::run(cmd, args),
        Commands::Child { cmd, args } => container::child(cmd, args),
    }
}