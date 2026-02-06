use clap::Args;
use minijinja::{Environment, Error as TemplateError};
use illuminate_string::Str;
use jiff::Zoned;
use jiff::fmt::strtime::format;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const MIGRATION_TEMPLATE: &str = include_str!("templates/migration.rs.j2");

#[derive(Debug)]
pub enum MigrationError {
    Template(TemplateError),
    Io(io::Error),
}

impl From<TemplateError> for MigrationError {
    fn from(err: TemplateError) -> Self {
        Self::Template(err)
    }
}

impl From<io::Error> for MigrationError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Args, Debug)]
#[command(about = "Create a new migration file")]
pub struct NewMigArgs {
    /// The name of the migration
    pub name: String,

    /// The table to be created
    #[arg(short = 't', long)]
    pub table: Option<String>,

    /// The table to migrate
    #[arg(short = 'c', long)]
    pub create: Option<String>,

    /// The location where the migration file should be created
    #[arg(short = 'p', long)]
    pub path: Option<String>,

    /// Indicate any provided migration file paths are pre-resolved absolute paths
    #[arg(long)]
    pub realpath: bool,
}

#[derive(serde::Serialize)]
struct MigrationContext {
    name: String,
    final_name: String,
    is_create: bool,
    is_table: bool,
    create: Option<String>,
    table: Option<String>,
}

pub fn migrate(args: &NewMigArgs) -> Result<bool, MigrationError> {
    let now = Zoned::now();

    let timestamp = format("%Y_%m_%d_%H_%M", &now)
        .unwrap_or_else(|_| "2025_01_01_00_00".into());

    let final_name = format!(
        "m_{}_{}",
        timestamp,
        Str::snake(&args.name, "_")
    );

    let mut env = Environment::new();
    env.add_template("migration", MIGRATION_TEMPLATE)?;

    let ctx = MigrationContext {
        name: Str::studly(&args.name),
        final_name: final_name.clone(),
        is_create: args.create.is_some(),
        is_table: args.table.is_some(),
        create: args.create.clone(),
        table: args.table.clone(),
    };

    let rendered = env.get_template("migration")?.render(ctx)?;

    // println!("{}", rendered);

    let file_name = format!("{}.rs", final_name);
    let target_path = resolve_target_path(&file_name, args)?;

    // println!("{}", target_path.display());

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&target_path, rendered)?;

    println!("{}",format!("migration file created: {}", final_name));

    Ok(true)
}

fn resolve_target_path(
    file_name: &str,
    args: &NewMigArgs,
) -> Result<PathBuf, io::Error> {
    match &args.path {
        None => {
            let base = std::env::current_dir()?.join("database/src/migrations");
            Ok(base.join(file_name))
        }

        Some(path) if args.realpath => {
            Ok(PathBuf::from(path).join(file_name))
        }

        Some(path) => {
            let cwd = std::env::current_dir()?;
            Ok(cwd.join(path).join(file_name))
        }
    }
}
