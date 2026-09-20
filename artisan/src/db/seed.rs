use clap::Args;
// use rustavel_db::

/// Arguments for the `seed` command.
///
/// `--class` allows a specific seeder to be executed instead of the
/// default database seeder.
#[derive(Args, Debug)]
pub struct SeedArgs {
    /// The name of the seeder class (struct) to run.
    ///
    /// Example:
    ///
    ///     migration seed --class UserSeeder
    #[arg(long = "class")]
    pub class: Option<String>,
}
pub fn seed(seed_args: SeedArgs){
    if seed_args.class.is_none(){
        // main is run all seeder

    }
}