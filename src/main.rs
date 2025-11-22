use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run { cmd: String, args: Vec<String> }, // User types this
    Child { cmd: String, args: Vec<String> } // Internal use only
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {cmd, args} => {
            println!("Parent: I need to start a container for '{}'", cmd);
            // TODO: Create isolation here
        }
        Commands::Child {cmd, args} => {
            println!("Child: I am inside the container running '{}'", cmd);
            // TODO: Become the shell here
        }
    }
}