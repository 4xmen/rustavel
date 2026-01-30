use clap::{Arg, Command};
use migrator::run_migrations;
use tokio::runtime::Runtime;

mod migrator;
mod migrations;

fn main() {

    dotenv::dotenv().ok();

    let app = Command::new("migration")
        .arg(
            Arg::new("down")
                .long("down")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let up = !app.get_flag("down");
        run_migrations(up).await.unwrap();
    });
}
