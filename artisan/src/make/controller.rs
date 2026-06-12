use std::time::Instant;
use clap::Args;
use illuminate_str::Str;
use minijinja::{ Environment};
use rustavel_core::facades::file_content::FileContent;
use rustavel_core::facades::terminal_ui::{operation, Status};
use crate::make::make_error::MakeError;
use std::path::Path;
use tokio::fs;

const CONTROLLER_TEMPLATE: &str = include_str!("templates/controller.rs.j2");
#[derive(Args, Debug)]
#[command(about = "Create a new controller file")]
pub struct NewControllerArgs {
    /// The name of the controller
    pub name: String,

    /// model name
    #[arg(long)]
    pub model: Option<String>,
}

#[derive(serde::Serialize)]
struct ControllerContext {
    model: String,
    has_model: bool,
}


pub async fn controller(args: &NewControllerArgs) -> Result<(), MakeError> {
    let start = Instant::now();
    let controller_name = Str::ucfirst( &Str::singular(&args.name) );


    let model_name = match &args.model {
        Some(model) => Str::ucfirst(model).to_string(),
        None=> "".to_string(),
    };

    let mut env = Environment::new();
    env.add_template("controller", CONTROLLER_TEMPLATE)?;

    let ctx = ControllerContext {
        model: model_name.clone(),
        has_model: model_name.len() > 0,
    };

    let rendered = env.get_template("controller")?.render(ctx)?;

    let base = std::env::current_dir()?.join("app/src/http/controllers");
    let generated_path = base.join(format!("{}.rs", Str::snake(&controller_name,"_")));
    
    // Write controller file
    FileContent::put(generated_path.to_str().unwrap(), &rendered).await?;

    // Register in mod.rs
    let mod_file = base.join("mod.rs");

    register_mod_file(&mod_file, &Str::snake(&controller_name, "_")).await?;

    operation(&format!("controller made: {:?}", controller_name),start.elapsed(),Status::Done);
    Ok(())
}


/// Ensure a module is registered in mod.rs (idempotent)
async fn register_mod_file(mod_file: &Path, module_name: &str) -> Result<(), MakeError> {
    // Read existing mod.rs content or create empty if missing
    let content = match fs::read_to_string(mod_file).await {
        Ok(c) => c,
        Err(_) => String::new(),
    };

    let line = format!("pub mod {};", module_name);

    // Avoid duplicate registration
    if content.lines().any(|l| l.trim() == line) {
        return Ok(());
    }

    // Append module declaration
    let mut new_content = content;
    if !new_content.ends_with('\n') && !new_content.is_empty() {
        new_content.push('\n');
    }
    new_content.push_str(&line);
    new_content.push('\n');

    fs::write(mod_file, new_content).await?;
    Ok(())
}