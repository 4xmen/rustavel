use std::time::Instant;
use clap::{Parser, Subcommand};
use rustavel_artisan::db::seed::SeedArgs;
use rustavel_core::facades::terminal_ui::{operation, title, Status, TitleKind};
use rustavel_core::logger;

mod factories;
mod migrator;
mod migrations;
mod seeders;


/// Root CLI definition.
///
/// Available commands:
///
///     cli-db migrate
///     cli-db migrate --fresh
///     cli-db migrate --fresh --seed
///     cli-db seed
///     cli-db seed --class UserSeeder
#[derive(Parser, Debug)]
#[command(
    name = "cli-db",
    version,
    about = "Database cli-db and seeding CLI"
)]
struct Cli {
    /// The command to execute.
    #[command(subcommand)]
    command: Command,
}


/// Available CLI commands.
///
/// cli-db and seeding are separate operations. However, the `migrate`
/// command can optionally run the default database seeder after cli-db.
#[derive(Subcommand, Debug)]
enum Command {
    /// Run database cli-dbs.
    ///
    /// Examples:
    ///
    ///     cli-db migrate
    ///     cli-db migrate --fresh
    ///     cli-db migrate --rollback 2
    ///     cli-db migrate --fresh --seed
    Migrate {
        /// Rollback the specified number of cli-db steps.
        #[arg(long, default_value_t = 0)]
        rollback: i64,

        /// Run cli-dbs in passive mode.
        #[arg(long)]
        passive: bool,

        /// Drop all tables and re-run all cli-dbs.
        ///
        /// Can be combined with `fresh`.
        #[arg(long)]
        fresh: bool,

        /// Run the default database seeder after cli-dbs.
        ///
        /// Example:
        ///
        ///     cli-db migrate fresh seed
        #[arg(long)]
        seed: bool,
    },

    /// Run database seeders independently from cli-dbs.
    ///
    /// Examples:
    ///
    ///     cli-db seed
    ///     cli-db seed --class UserSeeder
    Seed(SeedArgs),
}




/// Application entry point.
///
/// Tokio creates and manages the async runtime for the entire CLI.
/// This allows both cli-db and seeding operations to use `await`.
#[tokio::main]
async fn main() {
    // Load environment variables from `.env`, if available.
    dotenv::dotenv().ok();

    // Parse command-line arguments.
    let cli = Cli::parse();

    // Execute the selected command.
    if let Err(error) = run(cli).await {
        logger::error(&format!("{:?}", error));
    }
}


/// Execute the selected CLI command.
///
/// This function only handles command orchestration. The actual cli-db
/// and seeding logic lives inside their respective modules.
async fn run(cli: Cli) -> Result<(), anyhow::Error> {
    match cli.command {
        // -------------------------------------------------------------
        // MIGRATE
        // -------------------------------------------------------------
        //
        // cli-db always runs first. If `--seed` is specified, the
        // default DatabaseSeeder runs after cli-db completes.
        Command::Migrate {
            rollback,
            passive,
            fresh,
            seed,
        } => {
            // Run cli-dbs first. If this fails, seeding is skipped.
            migrator::run_migrations(rollback, passive, fresh).await?;

            // `migrate --seed` runs the default DatabaseSeeder.
            //
            // Unlike the standalone `seed` command, no specific seeder
            // class is selected here.
            if seed {
                let start = Instant::now();
                title(TitleKind::Info,"Running seeders");
                match seeders::database_seeder::DatabaseSeeder::run().await {
                    Ok(_e) =>{
                        operation("All seeder done", start.elapsed(), Status::Done);
                    }
                    Err(e) => {
                        operation(&format!("Seeding :{}", e), start.elapsed(), Status::Failed);
                    }
                }
            }
        }

        // -------------------------------------------------------------
        // SEED
        // -------------------------------------------------------------
        //
        // Seeding is completely independent from cli-db.
        //
        // Examples:
        //
        //     cli-db seed
        //     cli-db seed --class UserSeeder
        Command::Seed(_args) => {
            // The SeedArgs value can be used by DatabaseSeeder to decide
            // whether a specific seeder class should be executed.
            // print!("{:?}", args);
            seeders::database_seeder::DatabaseSeeder::run().await?;
        }
    }

    Ok(())
}
