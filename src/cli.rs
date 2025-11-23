// This file defines how users talk to your program. It only cares about arguments.
// We added pub (public) so other modules can see these structs.

use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Run { cmd: String, args: Vec<String> },
    Child { cmd: String, args: Vec<String> },
}