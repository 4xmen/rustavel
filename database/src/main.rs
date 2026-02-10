// use std::process::exit;
use clap::Parser;
use migrator::run_migrations;
use tokio::runtime::Runtime;
use rustavel_core::logger;

mod migrator;
mod migrations;

#[derive(Parser, Debug)]
#[command(name = "migration")]
struct Cli {
    /// rollback step count
    #[arg(long, default_value_t = 0)]
    rollback: i64,

    /// Run migrations in passive mode ( Effective just in up mode)
    #[arg(long)]
    passive: bool,

    ///  Drop all tables and re-run all migrations
    #[arg(long)]
    fresh: bool,
}


fn main() {
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    let rt = Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {

        println!("Running database migrations{}",cli.rollback);
        run_migrations(cli.rollback, cli.passive, cli.fresh)
            .await.unwrap_or_else(|e|{
            logger::error(&format!("{:?}", e));
        });
    });
}
