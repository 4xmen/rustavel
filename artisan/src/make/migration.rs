use clap::Args;
use minijinja::{Environment, Error as TemplateError};
use illuminate_string::Str;
use jiff::Zoned;
use jiff::fmt::strtime::format;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;
use crate::utility::ui::{title, operation, TitleKind, Status};

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


/// Create and persist a new migration file from CLI (artisan) input.
///
/// This function does:
/// 1. Generate a timestamped and snake_cased migration filename.
/// 2. Build a rendering context from CLI arguments (create/table flags, names).
/// 3. Render the migration source code using the migration template.
/// 4. Resolve the target filesystem path for the migration file.
/// 5. Create parent directories if they do not exist.
/// 6. Write the rendered migration to disk.
/// 7. Register the new migration in `database/src/migrations/mod.rs`.
/// 8. Report the operation status and execution time.
pub fn migrate(args: &NewMigArgs) -> Result<bool, MigrationError> {

    let start = Instant::now();

    let now = Zoned::now();

    let timestamp = format("%Y_%m_%d_%H%M", &now)
        .unwrap_or_else(|_| "2025_01_01_0000".into());

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


    // It measures execution time, not disk flush time.
    operation(
        &format!("migration created: {}", final_name),
        start.elapsed(),
        Status::Done,
    );

    register_new_migration(&final_name,&Str::studly(&args.name) )?;

    Ok(true)
}


/// Resolve the final filesystem path for the migration file.
///
/// This function does:
/// 1. Use the default `database/src/migrations` directory when no path is provided.
/// 2. Treat the provided path as an absolute path when `--realpath` is set.
/// 3. Otherwise, resolve the provided path relative to the current working directory.
/// 4. Append the migration filename to the resolved base path.
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



/// Update `mod.rs` to register a new migration.
///
/// This function does:
/// 1. Locate `mod.rs` inside `database/src/migrations`.
/// 2. Validate that placeholders exist:
///    - `// #[add-mig-mods]`
///    - `// #[add-mig-trait]`
/// 3. Check duplicates for module and struct.
/// 4. Append new module and struct before placeholders.
pub fn register_new_migration(final_name: &str, struct_raw: &str) -> io::Result<()> {
    // Derive module and struct names
    let module_name = final_name; // same as file name without `.rs`
    let struct_name = format!(
        "{}::{}",
        &final_name,
        &struct_raw
    );

    // Locate mod.rs
    let mod_rs_path: PathBuf = std::env::current_dir()?
        .join("database/src/migrations/mod.rs");

    // Read content
    let content = fs::read_to_string(&mod_rs_path)?;

    // Placeholders
    let mod_placeholder = "// #[add-mig-mods] DO NOT REMOVE THIS COMMENT, OTHERWISE AUTOMATIC ADD WILL BREAK";
    let trait_placeholder = "// #[add-mig-trait] DO NOT REMOVE THIS COMMENT, OTHERWISE AUTOMATIC ADD WILL BREAK";

    // Validate placeholders exist
    if !content.contains(mod_placeholder) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Placeholder '{}' not found in mod.rs", mod_placeholder),
        ));
    }
    if !content.contains(trait_placeholder) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Placeholder '{}' not found in mod.rs", trait_placeholder),
        ));
    }

    // Check duplicates
    if content.contains(&format!("pub mod {};", module_name)) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Module '{}' already exists in mod.rs", module_name),
        ));
    }
    if content.contains(&format!("Box::new({} {{}})", struct_name)) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Struct '{}' already exists in get_all_migrations", struct_name),
        ));
    }

    // Append module and struct before placeholders
    let new_content = content
        .replace(
            mod_placeholder,
            &format!("pub mod {};\n{}", module_name, mod_placeholder),
        )
        .replace(
            trait_placeholder,
            &format!("Box::new({} {{}}),\n        {}", struct_name, trait_placeholder),
        );

    // Write back to mod.rs
    fs::write(&mod_rs_path, new_content)?;

    // println!("✅ mod.rs updated: module '{}' registered", module_name);

    Ok(())
}
