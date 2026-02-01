use std::env::current_dir;
use clap::{Parser, Subcommand};
use std::process::Command as ProcessCommand;


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
            ProcessCommand::new("cargo")
                .args(["run", "--package", "rustavel-db", "--bin", "database"])
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
                Ok(s) => eprintln!("cargo watch با کد {} خارج شد", s.code().unwrap_or(-1)),
                Err(e) => eprintln!("نشد cargo watch رو اجرا کنیم: {}", e),
            }
        }
        // add another command here :)
    }
}
