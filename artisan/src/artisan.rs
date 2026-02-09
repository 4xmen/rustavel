mod make;
mod general;

// use std::env::current_dir;
use clap::{Parser, Subcommand};
use std::process::Command as ProcessCommand;
use rustavel_core::config::CONFIG;
// use clap::Args;
use crate::make::migration::{NewMigArgs,migrate};
use dialoguer::{theme::ColorfulTheme, Confirm};
use rustavel_core::facades::terminal_ui::{TitleKind, title};
fn confirm(message: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(false)
        .interact()
        .unwrap()
}
use crate::general::lib::{generate_laravel_app_key, set_env_value};

#[derive(Parser)]
#[command(name = "artisan")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    //// app key generate
    KeyGenerate,
    Migrate {
        /// Run migrations down
        #[arg(long)]
        down: bool,

        ///  Drop all tables and re-run all migrations
        #[arg(long)]
        fresh: bool,

        /// Run migrations in passive mode ( Effective just in up mode)
        #[arg(long)]
        passive: bool,

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

        Commands::KeyGenerate => {
            if !CONFIG.app.key.is_empty() {
                if !confirm("Are you sure you want to regenerate key?") {

                    title(TitleKind::Error,"Application key set successfully.");
                    std::process::exit(0);
                }
            }

            let app_key = generate_laravel_app_key();

            match set_env_value( "APP_KEY", &app_key) {
                Ok(_) => {
                    title(TitleKind::Info,"Cancelled...");
                }
                Err(e) => {
                    title(TitleKind::Error,&format!("failed to set APP_KEY: {}", e));
                }
            }

        }
        Commands::Migrate  { down, fresh, passive } => {

            if CONFIG.app.env == "production" {
                if !confirm("Are you sure you want to run migration in production mode?") {

                    title(TitleKind::Info,"Cancelled...");
                    std::process::exit(0);
                }
            }
            let mut args = vec!["run", "--package", "rustavel-db", "--bin", "database"];
            if down || fresh || passive {
                args.push("--");
            }
            if down {
                args.push("--down");
            }
            if fresh {
                args.push("--fresh");
            }
            if passive {
                args.push("--passive");
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
                Ok(s) => title(TitleKind::Success,&format!("cargo watch exit with code: {} ", s.code().unwrap_or(-1))),
                Err(e) => title(TitleKind::Error, &format!("cargo watch can't run: {}", e)),
            }
        },
        Commands::Make { kind } => {
            // println!("what use did? {:?}", kind);

            match kind {
                MakeCmd::Migration(args) => {
                    let _ = migrate(&args).unwrap_or_else(|e| {
                        println!("{:?}",e);
                        title(TitleKind::Error, &format!("migration error: {:?}", e));
                        false
                    });
                },
            }
        }
        // add another command here :)
    }
}
