use clap::Parser;
use migrator::run_migrations;
use tokio::runtime::Runtime;

mod migrator;
mod migrations;

#[derive(Parser, Debug)]
#[command(name = "migration")]
struct Cli {
    /// Run migrations downward
    #[arg(long)]
    down: bool,

    /// Run migrations in passive mode ( Effective just in up mode)
    #[arg(long)]
    passive: bool,
}

fn main() {
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    let rt = Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        let up = !cli.down;
        run_migrations(up, cli.passive)
            .await.unwrap_or_default();
    });
}
