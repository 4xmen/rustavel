use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(name = "artisan")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Migrate,
    Serv,
    // may add: Make { kind: String, name: String }, Seed, etc.
}

fn main() {
    dotenv::dotenv().ok();
    let cli = Cli::parse();

    match cli.command {
        Commands::Migrate => {
            // compile and run database
            Command::new("cargo")
                .args(["run", "--package", "database", "--bin", "database"])
                .status()
                .unwrap();
        }
        Commands::Serv => {
            // serv app
            Command::new("cargo")
                .args(["run", "--package", "rustavel-app", "--bin", "rustavel-app"])
                .status()
                .unwrap();
        } // add another command here :)
    }
}
