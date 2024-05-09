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
    Git { command: Vec<String> },
    /// Allow authentication in applicationhost.config
    ApplicationHost,
    /// Fix Azure auth in Web API appsettings
    WebApi,
    /// Create a new user in database, with an attached role
    CreateUser {
        name: String,
        email: String,
        connection_string: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ConnectionStrings {
            computer_name,
            main,
            service_bus,
        } => {
            commands::connection_strings::invoke(computer_name, main, service_bus, cli.path);
        }
        Commands::Git { command } => {
            commands::git_cmd::invoke(command, cli.path);
        }
        Commands::ApplicationHost => {
            commands::application_host::invoke(cli.path);
        }
        Commands::WebApi => {
            commands::web_api::invoke(cli.path);
        }
        Commands::CreateUser {
            name,
            email,
            connection_string,
        } => {
            commands::create_user::invoke(name, email, connection_string).await;
        }
    }
}
