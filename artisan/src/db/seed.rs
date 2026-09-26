use clap::Args;
// use rustavel_db::

/// Arguments for the `seed` command.
///
/// `--class` allows a specific seeder to be executed instead of the
/// default database seeder.
#[derive(Args, Debug)]
pub struct SeedArgs {
    /// The name of the seeder class (struct) to run.
    #[arg(long = "class")]
    pub class: Option<String>,
}