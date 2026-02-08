mod make;

// use std::env::current_dir;
use clap::{Parser, Subcommand};
use std::process::Command as ProcessCommand;
use rustavel_core::config::CONFIG;
// use clap::Args;
use crate::make::migration::{NewMigArgs,migrate};
use dialoguer::{theme::ColorfulTheme, Confirm};

fn confirm(message: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(false)
        .interact()
        .unwrap()
}

#[derive(Parser)]
#[command(name = "artisan")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Migrate {
        /// Run migrations down
        #[arg(long)]
        down: bool,

    },
    Serv,
    Make {
        #[command(subcommand)]
        kind: MakeCmd,
    },
    // may add: Make { kind: String, name: String }, Seed, etc.
}


#[derive(Subcommand, Debug)]
enum MakeCmd {
    /// Create a new migration file
    Migration(NewMigArgs),
}




fn main() {

    dotenv::dotenv().ok();
    let cli = Cli::parse();

    match cli.command {
        Commands::Migrate  { down} => {

            if CONFIG.app.env == "production" {
                if !confirm("Are you sure you want to run migration in production mode?") {

                    println!("cancelled...");
                    std::process::exit(0);
                }
            }
            let mut args = vec!["run", "--package", "rustavel-db", "--bin", "database"];
            if down {
                args.push("--");
                args.push("--down");
            }
            // compile and run database
            ProcessCommand::new("cargo")
                .args(args)
                .status()
                .unwrap();
        }
        Commands::Serv => {
            println!("Starting rustavel-app with hot-reload (cargo watch)...");

            let status = ProcessCommand::new("cargo")
                .args([
                    "watch",
                    "-p", "rustavel-app",
                    "--ignore", "target",
                    "-x", "run --package rustavel-app --bin rustavel-app",
                ])
                .status();

            match status {
                Ok(s) if s.success() => {},
                Ok(s) => eprintln!("cargo watch exit with code: {} ", s.code().unwrap_or(-1)),
                Err(e) => eprintln!(" cargo watch can't run: {}", e),
            }
        },
        Commands::Make { kind } => {
            // println!("what use did? {:?}", kind);

            match kind {
                MakeCmd::Migration(args) => {
                    let _ = migrate(&args).unwrap_or_else(|e| {
                        println!("{:?}",e);
                        false
                    });
                },
            }
        }
        // add another command here :)
    }
}
