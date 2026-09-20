use clap::Args;

#[derive(Args, Debug)]
struct SeedArgs {
    /// The name of the seeder class (struct) to run
    #[arg(long = "class")]
    class: Option<String>,
}