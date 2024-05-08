use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub(crate) mod commands;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[clap(short, long)]
    path: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sets your connection strings everywhere it needs to be set
    ConnectionStrings {
        computer_name: String,
        main: String,
        service_bus: String,
    },
    /// Run a command against each git repository
    GitCmd { command: String },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ConnectionStrings {
            computer_name,
            main,
            service_bus,
        } => {
            commands::connection_strings::invoke(computer_name, main, service_bus, cli.path);
        }
        Commands::GitCmd { command } => {
            commands::git_cmd::invoke(command);
        }
    }
}
